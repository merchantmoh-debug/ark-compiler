import sys
import json
import os
import logging

# Configure logging to stderr so it doesn't interfere with stdout (JSON-RPC)
logging.basicConfig(stream=sys.stderr, level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

try:
    from lark import Lark, exceptions
except ImportError:
    logging.error("Lark not found. Please install it.")
    sys.exit(1)

class ArkLSP:
    """
    A basic Language Server Protocol (LSP) stub for Ark.
    Handles 'initialize', 'textDocument/didOpen', and 'textDocument/didChange'.
    Uses raw JSON-RPC over stdio.
    """
    def __init__(self):
        # Determine the path to the grammar file relative to this script
        self.grammar_path = os.path.join(os.path.dirname(__file__), "ark.lark")
        try:
            with open(self.grammar_path, "r") as f:
                grammar = f.read()
            # Use LALR parser for performance, matching the interpreter implementation
            self.parser = Lark(grammar, start="start", parser="lalr")
            logging.info(f"Loaded grammar from {self.grammar_path}")
        except Exception as e:
            logging.error(f"Failed to load grammar: {e}")
            self.parser = None

    def run(self):
        """
        Main loop for the LSP server.
        Reads JSON-RPC messages from stdin and dispatches them.
        """
        logging.info("Ark LSP started")
        while True:
            headers = {}
            try:
                # Read headers
                while True:
                    line = sys.stdin.buffer.readline()
                    if not line: # EOF
                        return
                    line = line.decode('ascii').strip()
                    if not line: # End of headers (empty line)
                        break
                    parts = line.split(":", 1)
                    if len(parts) == 2:
                        headers[parts[0].strip()] = parts[1].strip()

                # Read content
                if "Content-Length" in headers:
                    length = int(headers["Content-Length"])
                    body = sys.stdin.buffer.read(length).decode('utf-8')
                    request = json.loads(body)
                    self.handle_message(request)
            except Exception as e:
                logging.error(f"Error in run loop: {e}")
                # Don't crash on bad message, just log and continue
                pass

    def handle_message(self, msg):
        """
        Dispatches the message to the appropriate handler.
        """
        method = msg.get("method")
        msg_id = msg.get("id")
        params = msg.get("params", {})

        logging.info(f"Received method: {method}")

        if method == "initialize":
            # Respond with capabilities
            result = {
                "capabilities": {
                    "textDocumentSync": 1 # Full sync
                }
            }
            self.send_response(msg_id, result)

        elif method == "shutdown":
            self.send_response(msg_id, None)

        elif method == "exit":
            sys.exit(0)

        elif method == "textDocument/didOpen":
            # Validate the document upon opening
            self.validate(params["textDocument"]["uri"], params["textDocument"]["text"])

        elif method == "textDocument/didChange":
            # Validate the document upon changes
            # Since we declared Full sync (1), we get the full text in the first change event
            if params.get("contentChanges"):
                text = params["contentChanges"][0]["text"]
                self.validate(params["textDocument"]["uri"], text)

    def validate(self, uri, text):
        """
        Validates the Ark code using the Lark parser and publishes diagnostics.
        """
        if not self.parser:
            return

        diagnostics = []
        try:
            self.parser.parse(text)
        except exceptions.UnexpectedToken as e:
            # Lark line/col are 1-based, LSP is 0-based
            line = e.line - 1
            col = e.column - 1

            # Try to determine token length for better highlighting
            length = 1
            if hasattr(e.token, "value"):
                length = len(e.token.value)

            diagnostics.append({
                "range": {
                    "start": {"line": line, "character": col},
                    "end": {"line": line, "character": col + length}
                },
                "severity": 1, # Error
                "message": f"Unexpected token: {e.token.value} (Expected: {', '.join(e.expected)})",
                "source": "ark-lsp"
            })
        except exceptions.UnexpectedCharacters as e:
            line = e.line - 1
            col = e.column - 1
            diagnostics.append({
                "range": {
                    "start": {"line": line, "character": col},
                    "end": {"line": line, "character": col + 1}
                },
                "severity": 1,
                "message": f"Unexpected characters: {e.char}",
                "source": "ark-lsp"
            })
        except Exception as e:
            logging.error(f"Validation error: {e}")
            pass

        # Publish diagnostics
        self.send_notification("textDocument/publishDiagnostics", {
            "uri": uri,
            "diagnostics": diagnostics
        })

    def send_response(self, msg_id, result):
        """
        Sends a JSON-RPC response.
        """
        response = {
            "jsonrpc": "2.0",
            "id": msg_id,
            "result": result
        }
        self.write_message(response)

    def send_notification(self, method, params):
        """
        Sends a JSON-RPC notification.
        """
        notification = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }
        self.write_message(notification)

    def write_message(self, msg):
        """
        Writes a message to stdout with Content-Length header.
        """
        body = json.dumps(msg)
        content_length = len(body.encode('utf-8'))
        response = f"Content-Length: {content_length}\r\n\r\n{body}"
        sys.stdout.buffer.write(response.encode('utf-8'))
        sys.stdout.buffer.flush()

if __name__ == "__main__":
    server = ArkLSP()
    server.run()
