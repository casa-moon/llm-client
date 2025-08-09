const templates = {
  openai(rawLog, client) {
    return rawLog.map(message => {
      const role = message.role === 'model' ? 'assistant' : message.role;
      let content = [];
      if (message.type === 'text') {
        content = [{ type: message.type, text: message.content }];
      } else if (message.type === 'image') {
        content = [{ type: 'image_url', image_url: { url: message.content } }];
      }
      return { role, content };
    });
  },

  anthropic(rawLog, client) {
    let transformed = [];
    let userContent = '';
    for (const message of rawLog) {
      if (message.role === 'user') {
        userContent += `<doc>${message.content}</doc>`;
      } else {
        if (userContent) {
          transformed.push({
            role: 'user',
            content: client.stripDocTagsIfOnlyOneSet(userContent)
          });
          userContent = '';
        }
        transformed.push({ role: 'assistant', content: message.content });
      }
    }
    if (userContent) {
      transformed.push({
        role: 'user',
        content: client.stripDocTagsIfOnlyOneSet(userContent)
      });
    }
    return transformed;
  },

  google(rawLog, client) {
    let transformed = [];
    let userContent = '';
    for (const message of rawLog) {
      if (message.role === 'user') {
        userContent += `<doc>${message.content}</doc>`;
      } else {
        if (userContent) {
          transformed.push({
            role: 'user',
            parts: [{ text: client.stripDocTagsIfOnlyOneSet(userContent) }]
          });
          userContent = '';
        }
        transformed.push({ role: 'model', parts: [{ text: message.content }] });
      }
    }
    if (userContent) {
      transformed.push({
        role: 'user',
        parts: [{ text: client.stripDocTagsIfOnlyOneSet(userContent) }]
      });
    }
    transformed.push({ role: 'model', parts: [{ text: 'continue' }] });
    return transformed;
  },

  mistral(rawLog) {
    let transformed = [];
    let userContent = '';
    for (const message of rawLog) {
      if (message.role === 'user') {
        userContent += `<doc>${message.content}</doc>`;
      } else {
        if (userContent) {
          transformed.push({ role: 'user', content: userContent });
          userContent = '';
        }
        transformed.push({ role: 'assistant', content: message.content });
      }
    }
    if (userContent) {
      transformed.push({ role: 'user', content: userContent });
    }
    return transformed;
  },

  perplexity(rawLog) {
    let transformed = [];
    let userContent = '';
    for (const message of rawLog) {
      if (message.role === 'user') {
        userContent += `<doc>${message.content}</doc>`;
      } else {
        if (userContent) {
          transformed.push({
            role: 'user',
            content: [{ type: 'text', text: userContent }]
          });
          userContent = '';
        }
        transformed.push({
          role: 'assistant',
          content: [{ type: message.type, text: message.content }]
        });
      }
    }
    if (userContent) {
      transformed.push({
        role: 'user',
        content: [{ type: 'text', text: userContent }]
      });
    }
    return transformed;
  },

  ollama(rawLog) {
    let transformed = [];
    let userContent = '';
    for (const message of rawLog) {
      if (message.role === 'user') {
        userContent += `<doc>${message.content}</doc>`;
      } else {
        if (userContent) {
          transformed.push({
            role: 'user',
            content: [{ type: 'text', text: userContent }]
          });
          userContent = '';
        }
        transformed.push({
          role: 'assistant',
          content: [{ type: message.type, text: message.content }]
        });
      }
    }
    if (userContent) {
      transformed.push({
        role: 'user',
        content: [{ type: 'text', text: userContent }]
      });
    }
    return transformed;
  }
};

module.exports = templates;
