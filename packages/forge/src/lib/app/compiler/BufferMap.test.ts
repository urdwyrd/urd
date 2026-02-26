import { describe, it, expect, beforeEach } from 'vitest';
import { BufferMap } from './BufferMap';

describe('BufferMap', () => {
  let bm: BufferMap;

  beforeEach(() => {
    bm = new BufferMap();
  });

  it('starts empty', () => {
    expect(bm.size).toBe(0);
    expect(bm.get('/foo.urd.md')).toBeUndefined();
  });

  it('set() stores content and marks dirty', () => {
    bm.set('/a.urd.md', 'hello');
    expect(bm.get('/a.urd.md')).toBe('hello');
    expect(bm.isDirty('/a.urd.md')).toBe(true);
  });

  it('load() stores content without marking dirty', () => {
    bm.load('/a.urd.md', 'hello');
    expect(bm.get('/a.urd.md')).toBe('hello');
    expect(bm.isDirty('/a.urd.md')).toBe(false);
  });

  it('set() is a no-op when content is identical', () => {
    bm.set('/a.urd.md', 'hello');
    const changes: string[] = [];
    bm.subscribe((path) => changes.push(path));

    bm.set('/a.urd.md', 'hello'); // same content
    expect(changes).toEqual([]);
  });

  it('getAll() returns all entries', () => {
    bm.load('/a.urd.md', 'aaa');
    bm.set('/b.urd.md', 'bbb');

    expect(bm.getAll()).toEqual({
      '/a.urd.md': 'aaa',
      '/b.urd.md': 'bbb',
    });
  });

  it('paths() returns file paths', () => {
    bm.load('/a.urd.md', 'a');
    bm.load('/b.urd.md', 'b');
    expect(bm.paths()).toEqual(['/a.urd.md', '/b.urd.md']);
  });

  it('remove() removes a buffer and notifies', () => {
    bm.load('/a.urd.md', 'a');
    const changes: string[] = [];
    bm.subscribe((path) => changes.push(path));

    bm.remove('/a.urd.md');
    expect(bm.get('/a.urd.md')).toBeUndefined();
    expect(changes).toEqual(['/a.urd.md']);
  });

  it('remove() is a no-op for non-existent paths', () => {
    const changes: string[] = [];
    bm.subscribe((path) => changes.push(path));
    bm.remove('/nope');
    expect(changes).toEqual([]);
  });

  it('markClean() clears dirty flag', () => {
    bm.set('/a.urd.md', 'hello');
    expect(bm.isDirty('/a.urd.md')).toBe(true);
    bm.markClean('/a.urd.md');
    expect(bm.isDirty('/a.urd.md')).toBe(false);
  });

  it('markAllClean() clears all dirty flags', () => {
    bm.set('/a.urd.md', 'a');
    bm.set('/b.urd.md', 'b');
    expect(bm.hasAnyDirty()).toBe(true);
    bm.markAllClean();
    expect(bm.hasAnyDirty()).toBe(false);
  });

  it('clear() removes all buffers', () => {
    bm.load('/a.urd.md', 'a');
    bm.load('/b.urd.md', 'b');
    bm.clear();
    expect(bm.size).toBe(0);
    expect(bm.getAll()).toEqual({});
  });

  it('subscribe() notifies on set()', () => {
    const changes: string[] = [];
    bm.subscribe((path) => changes.push(path));

    bm.set('/a.urd.md', 'hello');
    bm.set('/a.urd.md', 'world');

    expect(changes).toEqual(['/a.urd.md', '/a.urd.md']);
  });

  it('unsubscribe stops notifications', () => {
    const changes: string[] = [];
    const unsub = bm.subscribe((path) => changes.push(path));

    bm.set('/a.urd.md', 'hello');
    unsub();
    bm.set('/b.urd.md', 'world');

    expect(changes).toEqual(['/a.urd.md']);
  });
});
