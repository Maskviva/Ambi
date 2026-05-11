// bindings/node/lib/llama.d.ts
import {LLMEngineConfig} from './index.js';
import type {LlamaContextSequence} from 'node-llama-cpp';

export class NodeLlamaAdapter {
    static create(sequence: LlamaContextSequence): LLMEngineConfig;
}