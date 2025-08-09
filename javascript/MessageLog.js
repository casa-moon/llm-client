class MessageLog {
  constructor() {
    this.rawMessageLog = [];
  }

  addTempMessageLog(tempMessageLog) {
    this.rawMessageLog.push(...tempMessageLog);
  }

  addUserMessage(content) {
    this.rawMessageLog.push({
      role: 'user',
      type: 'text',
      content: content
    });
  }

  addModelMessage(content) {
    this.rawMessageLog.push({
      role: 'model',
      type: 'text',
      content: content
    });
  }

  getRawMessageLog() {
    return this.rawMessageLog;
  }

  clearMessageLog() {
    this.rawMessageLog = [];
  }
}

module.exports = MessageLog;