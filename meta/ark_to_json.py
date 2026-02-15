import sys
import os
import json
from lark import Transformer, Tree

# Add repo root to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from meta.ark import ARK_PARSER

class ArkMastCompiler(Transformer):
    def start(self, items):
        # Flatten items if needed
        flat_items = []
        for i in items:
            if isinstance(i, list): flat_items.extend(i)
            else: flat_items.append(i)

        stmts = []
        for item in flat_items:
            if item is None: continue
            # Wrap Expression in Statement
            if any(k in item for k in ["Call", "Variable", "Literal", "List", "StructInit", "GetField"]):
                 stmts.append({ "Expression": item })
            else:
                 stmts.append(item)

        return {
            "Statement": {
                "Block": stmts
            }
        }

    # -- Statements --
    def block(self, items):
        stmts = []
        # Flatten nested lists if any (Lark might nest blocks)
        flat_items = []
        for i in items:
            if isinstance(i, list): flat_items.extend(i)
            else: flat_items.append(i)

        for item in flat_items:
            if item is None:
                continue

            # Wrap Expression in Statement
            # item is a dict
            if any(k in item for k in ["Call", "Variable", "Literal", "List", "StructInit", "GetField"]):
                 stmts.append({ "Expression": item })
            else:
                 stmts.append(item)

        # If block is empty (e.g. only comments), return empty list
        return { "Block": stmts }

    def function_def(self, items):
        name = items[0].value
        params = []
        body_node = None

        # Iterate items to find params and body
        for item in items[1:]:
            if item is None: continue
            if isinstance(item, list) and (not item or isinstance(item[0], str)):
                params = item
            elif isinstance(item, dict) and "Block" in item:
                body_node = item
            elif isinstance(item, dict) and "Statement" in item:
                 body_node = { "Block": [item] }
            elif isinstance(item, dict) and "Expression" in item:
                 body_node = { "Block": [{"Statement": item}] } # Wrap expr

        if body_node is None:
            body_node = { "Block": [] }

        # Filter None/Null from body block
        if "Block" in body_node:
             body_node["Block"] = [s for s in body_node["Block"] if s is not None]

        body_mast = {
            "hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "content": { "Statement": body_node }
        }

        inputs = [[p, { "Shared": "Any" }] for p in params]

        return {
            "Function": {
                "name": name,
                "inputs": inputs,
                "output": { "Shared": "Any" },
                "body": body_mast
            }
        }

    def param_list(self, items):
        return [t.value for t in items]

    def flow_stmt(self, items):
        return items[0]

    def return_stmt(self, items):
        val = items[0] if items else { "Literal": "unit" } # Unit? Or { "Expression": ... }
        # Return expects Expression
        return { "Return": val }

    def if_stmt(self, items):
        cond = items[0]
        then_block = items[1] # block returns {"Block": [...]}

        # Core AST expects `Vec<Statement>`.
        then_stmts = then_block["Block"]

        else_block = None
        if len(items) > 2:
            else_item = items[2]
            if else_item is not None:
                if "Block" in else_item:
                    else_block = else_item["Block"]
                elif "If" in else_item: # Else If
                    # Wrap nested If in a block?
                    # Core AST `else_block` is `Option<Vec<Statement>>`.
                    # So we wrap the If statement in a list.
                    else_block = [{ "Statement": else_item }]

        return {
            "If": {
                "condition": cond,
                "then_block": then_stmts,
                "else_block": else_block
            }
        }

    def while_stmt(self, items):
        cond = items[0]
        body = items[1]["Block"]
        return {
            "While": {
                "condition": cond,
                "body": body
            }
        }

    def assign_var(self, items):
        name = items[0].value
        val = items[1]
        return {
            "Let": {
                "name": name,
                "ty": None,
                "value": val
            }
        }

    def assign_destructure(self, items):
        # "let" "(" IDENTIFIER ("," IDENTIFIER)* ")" ":=" expression
        names = [t.value for t in items[:-1]]
        val = items[-1]
        return {
            "LetDestructure": {
                "names": names,
                "value": val
            }
        }

    def assign_attr(self, items):
        # atom "." IDENTIFIER ":=" expression
        # items: [atom, field_name, value]
        obj = items[0]
        field = items[1].value
        val = items[2]

        # Core `SetField` takes `obj_name: String`.
        if "Variable" in obj:
            return {
                "SetField": {
                    "obj_name": obj["Variable"],
                    "field": field,
                    "value": val
                }
            }
        else:
            raise Exception("SetField only supported on variables")

    # -- Expressions --
    def expression(self, items): return items[0] # wrapper

    def number(self, items):
        return { "Literal": items[0].value } # String repr of number

    def string(self, items):
        return { "Literal": items[0].value[1:-1] } # Strip quotes

    def var(self, items):
        return { "Variable": items[0].value }

    def list_cons(self, items):
        content = items[0] if items else []
        if content is None: content = []
        return { "List": content }

    def struct_init(self, items):
        fields = items[0] if items else []
        if fields is None: fields = []
        return { "StructInit": { "fields": fields } }

    def field_list(self, items): return items
    def field_init(self, items): return [items[0].value, items[1]]

    def expr_list(self, items): return items

    def call_expr(self, items):
        func = items[0] # atom
        args = items[1] if len(items) > 1 else []
        if isinstance(args, list) and len(args) == 0: pass # empty args

        # Check if func is intrinsic (Variable)
        func_hash = ""
        if "Variable" in func:
             func_hash = func["Variable"]
        elif "GetField" in func:
             parts = []
             curr = func
             while "GetField" in curr:
                 parts.append(curr["GetField"]["field"])
                 curr = curr["GetField"]["obj"]

             if "Variable" in curr:
                 parts.append(curr["Variable"])
                 parts.reverse()
                 func_hash = ".".join(parts)
             else:
                 raise Exception(f"Indirect call not fully supported: {func}")
        else:
             # Handle indirect call? Core AST expects `function_hash: String`.
             # So we assume it's a known function name.
             raise Exception(f"Indirect call not fully supported: {func}")

        # Map operators if needed, but here we assume function calls
        return {
            "Call": {
                "function_hash": func_hash,
                "args": args
            }
        }

    def get_attr(self, items):
        obj = items[0]
        field = items[1].value
        # Core `GetField` expects `obj: Box<Expression>`.
        return {
            "GetField": {
                "obj": obj,
                "field": field
            }
        }

    def get_item(self, items):
        raise Exception("Index access [] not supported directly, use sys.list.get")

    # -- BinOps --
    def _binop(self, op, items):
        return {
            "Call": {
                "function_hash": op,
                "args": [items[0], items[1]]
            }
        }

    def add(self, items): return self._binop("intrinsic_add", items)
    def sub(self, items): return self._binop("intrinsic_sub", items)
    def mul(self, items): return self._binop("intrinsic_mul", items)
    def div(self, items): return self._binop("intrinsic_div", items)
    def mod(self, items): return self._binop("intrinsic_mod", items)
    def lt(self, items): return self._binop("intrinsic_lt", items)
    def gt(self, items): return self._binop("intrinsic_gt", items)
    def le(self, items): return self._binop("intrinsic_le", items)
    def ge(self, items): return self._binop("intrinsic_ge", items)
    def eq(self, items): return self._binop("intrinsic_eq", items)
    def neq(self, items):
        # neq is `not (eq a b)`
        eq = self._binop("intrinsic_eq", items)
        # Wrap expression in Statement if needed? No, Call args are Expressions.
        # But wait, `eq` is a dict { "Call": ... }.
        # This is already an Expression representation.
        return {
            "Call": {
                "function_hash": "intrinsic_not",
                "args": [eq]
            }
        }

    def logical_or(self, items):
        # items: [left, OR_token, right]
        return self._binop("intrinsic_or", [items[0], items[2]])

    def logical_and(self, items):
        # items: [left, AND_token, right]
        return self._binop("intrinsic_and", [items[0], items[2]])

    # Helper for structure
    # The Transformer returns Dicts.
    # ArkNode expects specific enum wrappers.
    # e.g. Statement -> { "Let": ... }
    # Expression -> { "Call": ... }
    # My return values above are mixed.
    # I should wrap them.

    def statement(self, items):
        item = items[0]
        # Check if item is an Expression variant
        if any(k in item for k in ["Call", "Variable", "Literal", "List", "StructInit", "GetField"]):
             return { "Expression": item }

        # Already a Statement variant (Let, LetDestructure, SetField, Return, Block, If, While, Function)
        return item

    def atom(self, items):
         # atoms are Expressions.
         # items[0] is the dict.
         if len(items) == 1: return items[0]
         return items[0]

    def primary(self, items):
        # Parens case: (expr)
        return items[0]

def compile_ark(source_path, output_path):
    with open(source_path, 'r') as f:
        source = f.read()

    tree = ARK_PARSER.parse(source)
    compiler = ArkMastCompiler()
    mast = compiler.transform(tree)

    # Canonical JSON
    import hashlib
    try:
        canonical = json.dumps(mast, sort_keys=True, separators=(',', ':'))
    except TypeError as e:
        print(f"Serialization Error: {e}")
        print("MAST Structure containing Tree:")
        def find_tree(node, path=""):
            if isinstance(node, Tree):
                print(f"Found Tree at {path}: {node.data}")
            elif isinstance(node, list):
                for i, item in enumerate(node):
                    find_tree(item, f"{path}[{i}]")
            elif isinstance(node, dict):
                for k, v in node.items():
                    find_tree(v, f"{path}.{k}")
        find_tree(mast)
        raise e

    # Output only the content (ArkNode) as expected by load_ark_program
    # The loader will compute the hash itself.
    with open(output_path, 'w') as f:
        json.dump(mast, f, indent=2)

    print(f"Compiled {source_path} to {output_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: ark_to_json.py <input.ark> <output.json>")
    else:
        compile_ark(sys.argv[1], sys.argv[2])
