const assert = require('node:assert/strict');
const { test } = require('node:test');
const ApiClient = require('../ApiClient');

const sampleLog = [
  { role: 'user', type: 'text', content: 'Hello' },
  { role: 'model', type: 'text', content: 'Hi' },
  { role: 'user', type: 'text', content: 'How are you?' }
];

test('openai template', () => {
  const client = new ApiClient('', null, 'openai');
  const result = client.transformMessageLog(sampleLog);
  assert.deepStrictEqual(result, [
    { role: 'user', content: [{ type: 'text', text: 'Hello' }] },
    { role: 'assistant', content: [{ type: 'text', text: 'Hi' }] },
    { role: 'user', content: [{ type: 'text', text: 'How are you?' }] }
  ]);
});

test('anthropic template', () => {
  const client = new ApiClient('', null, 'anthropic');
  const result = client.transformMessageLog(sampleLog);
  assert.deepStrictEqual(result, [
    { role: 'user', content: 'Hello' },
    { role: 'assistant', content: 'Hi' },
    { role: 'user', content: 'How are you?' }
  ]);
});

test('google template', () => {
  const client = new ApiClient('', null, 'google');
  const result = client.transformMessageLog(sampleLog);
  assert.deepStrictEqual(result, [
    { role: 'user', parts: [{ text: 'Hello' }] },
    { role: 'model', parts: [{ text: 'Hi' }] },
    { role: 'user', parts: [{ text: 'How are you?' }] },
    { role: 'model', parts: [{ text: 'continue' }] }
  ]);
});

test('mistral template', () => {
  const client = new ApiClient('', null, 'mistral');
  const result = client.transformMessageLog(sampleLog);
  assert.deepStrictEqual(result, [
    { role: 'user', content: '<doc>Hello</doc>' },
    { role: 'assistant', content: 'Hi' },
    { role: 'user', content: '<doc>How are you?</doc>' }
  ]);
});

test('perplexity template', () => {
  const client = new ApiClient('', null, 'perplexity');
  const result = client.transformMessageLog(sampleLog);
  assert.deepStrictEqual(result, [
    { role: 'user', content: [{ type: 'text', text: 'Hello' }] },
    { role: 'assistant', content: [{ type: 'text', text: 'Hi' }] },
    { role: 'user', content: [{ type: 'text', text: 'How are you?' }] }
  ]);
});

test('ollama template', () => {
  const client = new ApiClient('', null, 'ollama');
  const result = client.transformMessageLog(sampleLog);
  assert.deepStrictEqual(result, [
    { role: 'user', content: 'Hello' },
    { role: 'assistant', content: 'Hi' },
    { role: 'user', content: 'How are you?' }
  ]);
});
