export class TerminalElement extends HTMLElement {
    constructor() {
        super();
        this.history = [];
    }

    connectedCallback() {
        this.innerHTML = `
            <div class="glass-panel" style="height: 400px; display: flex; flex-direction: column;">
                <div id="output" style="flex-grow: 1; overflow-y: auto; white-space: pre-wrap; margin-bottom: 10px;">
                    Welcome to Ark v1.0.0
                    Type (help) for commands.
                    Initialize Neural Link... [OFFLINE - WASM ONLY]
                </div>
                <div style="display: flex;">
                    <span>> </span>
                    <input type="text" id="input" style="
                        background: transparent;
                        border: none;
                        color: var(--text-color);
                        font-family: var(--font-mono);
                        flex-grow: 1;
                        outline: none;
                        margin-left: 5px;
                    " autofocus>
                </div>
            </div>
        `;

        this.input = this.querySelector('#input');
        this.output = this.querySelector('#output');

        this.input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                const cmd = this.input.value;
                this.log(`> ${cmd}`);
                this.input.value = '';
                this.handleCommand(cmd);
            }
        });
    }

    log(text) {
        this.output.innerText += `\n${text}`;
        this.output.scrollTop = this.output.scrollHeight;
    }

    async handleCommand(cmd) {
        // Simple client-side mock for now, real WASM hook later
        if (cmd === '(help)') {
            this.log('Available: (print "msg"), (help)');
        } else if (cmd.startsWith('(print')) {
            const msg = cmd.match(/"(.*?)"/);
            if (msg) this.log(msg[1]);
            else this.log('Error: Invalid syntax');
        } else {
            // Forward to WASM Runtime (Placeholder)
            if (window.ark_eval) {
                try {
                    const res = window.ark_eval(cmd);
                    this.log(res);
                } catch (e) {
                    this.log(`Runtime Error: ${e}`);
                }
            } else {
                this.log('Error: Runtime not loaded.');
            }
        }
    }
}

customElements.define('ark-terminal', TerminalElement);
