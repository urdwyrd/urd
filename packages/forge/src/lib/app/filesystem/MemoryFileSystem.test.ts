import { describe, it, expect, beforeEach } from 'vitest';
import { MemoryFileSystem } from './MemoryFileSystem';

describe('MemoryFileSystem', () => {
  let fs: MemoryFileSystem;

  beforeEach(() => {
    fs = new MemoryFileSystem();
  });

  it('reads a seeded file', async () => {
    fs.seed({ '/project/main.urd.md': 'hello world' });
    const content = await fs.readFile('/project/main.urd.md');
    expect(content).toBe('hello world');
  });

  it('throws when reading a non-existent file', async () => {
    await expect(fs.readFile('/nope')).rejects.toThrow('File not found');
  });

  it('writes and reads back a file', async () => {
    await fs.writeFile('/project/new.urd.md', 'content');
    const result = await fs.readFile('/project/new.urd.md');
    expect(result).toBe('content');
  });

  it('lists directory entries', async () => {
    fs.seed({
      '/project/a.urd.md': 'aaa',
      '/project/b.urd.md': 'bbb',
    });

    const entries = await fs.listDirectory('/project');
    expect(entries.map((e) => e.name).sort()).toEqual(['a.urd.md', 'b.urd.md']);
    expect(entries.every((e) => e.isFile)).toBe(true);
  });

  it('stat returns file info', async () => {
    fs.seed({ '/project/test.txt': 'hello' });
    const info = await fs.stat('/project/test.txt');
    expect(info.size).toBe(5);
    expect(info.isFile).toBe(true);
    expect(info.isDirectory).toBe(false);
  });

  it('stat throws for non-existent path', async () => {
    await expect(fs.stat('/nope')).rejects.toThrow('Path not found');
  });

  it('exists() returns true for existing files', async () => {
    fs.seed({ '/project/a.txt': 'x' });
    expect(await fs.exists('/project/a.txt')).toBe(true);
    expect(await fs.exists('/project/nope.txt')).toBe(false);
  });

  it('mkdir creates a directory', async () => {
    await fs.mkdir('/project/subdir');
    expect(await fs.exists('/project/subdir')).toBe(true);
    const info = await fs.stat('/project/subdir');
    expect(info.isDirectory).toBe(true);
  });

  it('mkdir is idempotent', async () => {
    await fs.mkdir('/project/subdir');
    await fs.mkdir('/project/subdir');
    expect(await fs.exists('/project/subdir')).toBe(true);
  });

  it('seed creates parent directories', async () => {
    fs.seed({ '/a/b/c/file.txt': 'x' });
    expect(await fs.exists('/a')).toBe(true);
    expect(await fs.exists('/a/b')).toBe(true);
    expect(await fs.exists('/a/b/c')).toBe(true);
  });

  it('file watcher is notified on write', async () => {
    fs.seed({ '/project/test.txt': 'original' });

    const events: string[] = [];
    await fs.watchFile('/project/test.txt', (event) => {
      events.push(event.kind);
    });

    await fs.writeFile('/project/test.txt', 'updated');
    expect(events).toEqual(['modify']);
  });

  it('directory watcher is notified on child write', async () => {
    fs.seedDirectory('/project');

    const events: string[] = [];
    await fs.watchDirectory('/project', (event) => {
      events.push(`${event.kind}:${event.path}`);
    });

    await fs.writeFile('/project/new.txt', 'content');
    expect(events).toEqual(['create:/project/new.txt']);
  });

  it('unwatch stops notifications', async () => {
    fs.seed({ '/project/test.txt': 'x' });

    const events: string[] = [];
    const unwatch = await fs.watchFile('/project/test.txt', (event) => {
      events.push(event.kind);
    });

    await fs.writeFile('/project/test.txt', 'a');
    unwatch();
    await fs.writeFile('/project/test.txt', 'b');

    expect(events).toEqual(['modify']);
  });

  it('normalises backslash paths', async () => {
    fs.seed({ '/project/test.txt': 'hello' });
    const content = await fs.readFile('\\project\\test.txt');
    expect(content).toBe('hello');
  });
});
