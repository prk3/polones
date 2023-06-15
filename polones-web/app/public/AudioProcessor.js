class PolonesAudioProcessor extends AudioWorkletProcessor {
  constructor(...args) {
    super(...args);
    this.buffer = new Uint16RingBuffer(1_700_000 * 10);
    this.port.onmessage = (e) => {
      this.buffer.pushBack(e.data);
    };
  }
  process(inputs, outputs, parameters) {
    const channel = outputs[0][0];
    const outSamplesLength = channel.length;
    if (this.buffer.length === 0) {
      for (let i = 0; i < outSamplesLength; i++) {
        channel[i] = 0;
      }
    }
    const inSamplesLength = Math.min(this.buffer.length, Math.floor(outSamplesLength * 60 * 29780.5 / 44_100));
    for (let i = 0; i < outSamplesLength; i++) {
      channel[i] = this.buffer.at(Math.floor(i / outSamplesLength * inSamplesLength)) / 65536;
    }

    this.buffer.removeFront(inSamplesLength);

    return true;
  }
}

class RingBuffer {
  constructor(capacity) {
    this.start = 0;
    this.length = 0;
    this.buffer = new ArrayBuffer(capacity);
  }
  pushBack(data) {
    let bytesCopied = 0;
    while (bytesCopied < data.byteLength) {
      const end = this.start + this.length % this.buffer.byteLength;
      const copyLength = Math.min(data.byteLength - bytesCopied, this.buffer.byteLength - end);
      new Uint8Array(this.buffer, end, copyLength).set(new Uint8Array(data, bytesCopied, copyLength), 0);
      bytesCopied += copyLength;
      this.start = (this.start + Math.max(0, copyLength - this.buffer.byteLength + this.length)) % this.buffer.byteLength;
      this.length = Math.min(this.length + copyLength, this.buffer.byteLength);
    }
  }
  removeFront(length) {
    const trueLengthChange = Math.min(length, this.length);
    this.length -= trueLengthChange;
    this.start = (this.start + trueLengthChange) % this.buffer.byteLength;
  }
  at(index) {
    return new Uint8Array(this.buffer)[(this.start + index) % this.buffer.byteLength];
  }
}

class Uint16RingBuffer {
  constructor(capacity) {
    this.buffer = new RingBuffer(capacity * 2);
    this.length = 0;
  }
  pushBack(data) {
    this.buffer.pushBack(data.buffer);
    this.length = this.buffer.length / 2;
  }
  removeFront(length) {
    this.buffer.removeFront(length * 2);
    this.length = this.buffer.length / 2;
  }
  at(index) {
    return new Uint16Array(this.buffer.buffer)[(this.buffer.start / 2 + index) % (this.buffer.buffer.byteLength / 2)];
  }
}

registerProcessor("polones-audio-processor", PolonesAudioProcessor);
