import React from 'react';
import { render, screen } from '@testing-library/react';
import App from './App';

test('renders learn react link', () => {

  class RingBuffer {
    start: number;
    length: number;
    buffer: ArrayBuffer;

    constructor(capacity: number) {
      this.start = 0;
      this.length = 0;
      this.buffer = new ArrayBuffer(capacity);
    }
    pushBack(data: ArrayBuffer) {
      let bytesCopied = 0;
      while (bytesCopied < data.byteLength) {
        let end = this.start + this.length % this.buffer.byteLength;
        let copyLength = Math.min(data.byteLength - bytesCopied, this.buffer.byteLength - end);
        new Uint8Array(this.buffer, end, copyLength).set(new Uint8Array(data, bytesCopied, copyLength), 0);
        bytesCopied += copyLength;
        this.start = (this.start + Math.max(0, copyLength - this.buffer.byteLength + this.length)) % this.buffer.byteLength;
        this.length = Math.min(this.length + copyLength, this.buffer.byteLength);
      }
    }
    removeFront(length: number) {
      let trueLengthChange = Math.min(length, this.length);
      this.length -= trueLengthChange;
      this.start = (this.start + trueLengthChange) % this.buffer.byteLength;
    }
    at(index: number) {
      return new Uint8Array(this.buffer)[(this.start + index) % this.buffer.byteLength];
    }
  }

  class Uint16RingBuffer {
    buffer: RingBuffer;

    constructor(capacity: number) {
      this.buffer = new RingBuffer(capacity * 2);
    }
    pushBack(data: Uint16Array) {
      this.buffer.pushBack(data.buffer);
    }
    removeFront(length: number) {
      this.buffer.removeFront(length * 2);
    }
    at(index: number) {
      return new Uint16Array(this.buffer.buffer)[(this.buffer.start / 2 + index) % (this.buffer.buffer.byteLength / 2)];
    }
  }

  let a;
  a = new RingBuffer(5);
  a.pushBack(new Uint8Array([1, 2, 3]).buffer);
  expect(a.start).toBe(0);
  expect(a.length).toBe(3);
  expect(a.buffer).toStrictEqual(new Uint8Array([1, 2, 3, 0, 0]).buffer);
  expect(a.at(0)).toBe(1);
  expect(a.at(2)).toBe(3);

  a.pushBack(new Uint8Array([4]).buffer);
  expect(a.start).toBe(0);
  expect(a.length).toBe(4);
  expect(a.buffer).toStrictEqual(new Uint8Array([1, 2, 3, 4, 0]).buffer);
  expect(a.at(0)).toBe(1);
  expect(a.at(3)).toBe(4);

  a.pushBack(new Uint8Array([5]).buffer);
  expect(a.start).toBe(0);
  expect(a.length).toBe(5);
  expect(a.buffer).toStrictEqual(new Uint8Array([1, 2, 3, 4, 5]).buffer);
  expect(a.at(0)).toBe(1);
  expect(a.at(4)).toBe(5);

  a.pushBack(new Uint8Array([6]).buffer);
  expect(a.start).toBe(1);
  expect(a.length).toBe(5);
  expect(a.buffer).toStrictEqual(new Uint8Array([6, 2, 3, 4, 5]).buffer);
  expect(a.at(0)).toBe(2);
  expect(a.at(4)).toBe(6);

  a.pushBack(new Uint8Array([7, 8]).buffer);
  expect(a.start).toBe(3);
  expect(a.length).toBe(5);
  expect(a.buffer).toStrictEqual(new Uint8Array([6, 7, 8, 4, 5]).buffer);
  expect(a.at(0)).toBe(4);
  expect(a.at(4)).toBe(8);

  a.pushBack(new Uint8Array([9, 10, 11, 12, 13, 14]).buffer);
  expect(a.start).toBe(4);
  expect(a.length).toBe(5);
  expect(a.buffer).toStrictEqual(new Uint8Array([11, 12, 13, 14, 10]).buffer);
  expect(a.at(0)).toBe(10);
  expect(a.at(4)).toBe(14);

  a.pushBack(new Uint8Array([15, 16, 17, 18, 19, 20, 21, 22]).buffer);
  expect(a.start).toBe(2);
  expect(a.length).toBe(5);
  expect(a.buffer).toStrictEqual(new Uint8Array([21, 22, 18, 19, 20]).buffer);
  expect(a.at(0)).toBe(18);
  expect(a.at(4)).toBe(22);

  a.removeFront(4);
  expect(a.start).toBe(1);
  expect(a.length).toBe(1);
  expect(a.buffer).toStrictEqual(new Uint8Array([21, 22, 18, 19, 20]).buffer);
  expect(a.at(0)).toBe(22);

  a.removeFront(2);
  expect(a.start).toBe(2);
  expect(a.length).toBe(0);
  expect(a.buffer).toStrictEqual(new Uint8Array([21, 22, 18, 19, 20]).buffer);
});
