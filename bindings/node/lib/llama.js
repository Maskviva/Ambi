import {LLMEngineConfig, resolveRequest} from './index.js';
import {LlamaCompletion} from "node-llama-cpp";

class NodeLlamaAdapter {
    static create(sequence) {
        const completion = new LlamaCompletion({contextSequence: sequence});

        return LLMEngineConfig.custom(
            // Synchronous callback — Rust's ThreadsafeFunction cannot await JS Promises,
            // so we start async work here and send the result back via resolveRequest().
            (err, reqJson) => {
                if (err) throw err;
                const payload = JSON.parse(reqJson);
                const {request_id, request} = payload;

                // Kick off the async LLM completion without awaiting — the result will
                // be sent back to Rust via resolveRequest() when done.
                (async () => {
                    try {
                        const response = await completion.generateCompletion(request.formatted_prompt, {
                            onTextChunk: (chunk) => {
                                process.stdout.write(chunk);
                            }
                        });
                        resolveRequest(request_id, response);
                    } catch (e) {
                        console.error("[LLM callback error]", e);
                    }
                })();
            },
            false,
            // Stream handler (same pattern, fire-and-forget, resolve via resolveRequest)
            (err, reqJson) => {
                if (err) throw err;
                const payload = JSON.parse(reqJson);
                const {request_id, request} = payload;

                (async () => {
                    try {
                        const response = await completion.generateCompletion(request.formatted_prompt, {
                            onTextChunk: (chunk) => {
                                process.stdout.write(chunk);
                            }
                        });
                        resolveRequest(request_id, response);
                    } catch (e) {
                        console.error("[LLM stream callback error]", e);
                    }
                })();
            }
        );
    }
}

export {NodeLlamaAdapter};
