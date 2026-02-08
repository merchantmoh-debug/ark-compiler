// Sovereign S-Expression Compiler (JS -> MAST)
// This proves the language is real by compiling text to JSON Architecture on the fly.

export class ArkCompiler {
    compile(source) {
        const tokens = this.tokenize(source);
        const ast = this.parse(tokens);
        return this.toMAST(ast);
    }

    tokenize(str) {
        return str.replace(/\(/g, ' ( ').replace(/\)/g, ' ) ')
                  .match(/".*?"|\S+/g) || [];
    }

    parse(tokens) {
        if (tokens.length === 0) return null;
        const token = tokens.shift();
        if (token === '(') {
            const list = [];
            while (tokens[0] !== ')') {
                list.push(this.parse(tokens));
            }
            tokens.shift(); // pop ')'
            return list;
        } else if (token === ')') {
            throw new Error("Unexpected )");
        } else {
            return this.atom(token);
        }
    }

    atom(token) {
        if (!isNaN(token)) return { type: 'int', val: token };
        if (token.startsWith('"')) return { type: 'str', val: token.slice(1, -1) };
        return { type: 'sym', val: token };
    }

    toMAST(ast) {
        // [print, "msg"] -> { "Expression": { "Call": { ... } } }
        if (Array.isArray(ast)) {
            const head = ast[0];
            
            // 1. Keywords
            if (head.val === 'print') {
                return {
                    "Statement": {
                        "Expression": {
                            "Call": {
                                "function_hash": "intrinsic_print",
                                "args": [this.expr(ast[1])]
                            }
                        }
                    }
                };
            }
            
            if (head.val === 'let') {
                 // (let x 10)
                 return {
                     "Statement": {
                         "Let": {
                             "name": ast[1].val,
                             "ty": null,
                             "value": this.expr(ast[2])
                         }
                     }
                 };
            }

            if (head.val === 'if') {
                // (if cond then else)
                return {
                    "Statement": {
                        "If": {
                            "condition": this.expr(ast[1]),
                            "then_block": [this.stmt(ast[2])],
                            "else_block": ast[3] ? [this.stmt(ast[3])] : null
                        }
                    }
                };
            }

            // Default: Function Call
            // (add 1 2)
            return {
                "Expression": {
                    "Call": {
                        "function_hash": this.resolveName(head.val),
                        "args": ast.slice(1).map(a => this.expr(a))
                    }
                }
            };
        }
        return this.expr(ast);
    }

    stmt(node) {
        const mast = this.toMAST(node);
        if (mast.Statement) return mast.Statement;
        // Wrap expression in statement
        if (mast.Expression) return { "Expression": mast.Expression };
        throw new Error("Invalid Statement");
    }

    expr(node) {
        if (node.type === 'int') return { "Literal": node.val };
        if (node.type === 'str') return { "Literal": node.val };
        if (node.type === 'sym') return { "Variable": node.val };
        
        // It's a list (Call)
        const mast = this.toMAST(node);
        if (mast.Expression) return mast.Expression;
         throw new Error("Expected Expression");
    }

    resolveName(name) {
        const map = {
            '+': 'intrinsic_add',
            '-': 'intrinsic_sub',
            '*': 'intrinsic_mul',
            '<': 'intrinsic_lt',
            '>': 'intrinsic_gt'
        };
        return map[name] || name;
    }
}
