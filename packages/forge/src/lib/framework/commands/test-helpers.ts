/**
 * TestCommandRegistry: records executions and undo actions.
 */

import { CommandRegistry, type CommandExecution, type UndoAction } from './CommandRegistry';

export interface RecordedExecution {
  commandId: string;
  args: unknown[];
  timestamp: number;
  undoAction: UndoAction | void;
}

export class TestCommandRegistry extends CommandRegistry {
  readonly executions: RecordedExecution[] = [];

  override async execute(commandId: string, ...args: unknown[]): Promise<UndoAction | void> {
    const result = await super.execute(commandId, ...args);
    this.executions.push({
      commandId,
      args,
      timestamp: Date.now(),
      undoAction: result,
    });
    return result;
  }

  executionsFor(commandId: string): RecordedExecution[] {
    return this.executions.filter((e) => e.commandId === commandId);
  }

  assertExecuted(commandId: string, times = 1): void {
    const actual = this.executionsFor(commandId).length;
    if (actual !== times) {
      throw new Error(`Expected command "${commandId}" to execute ${times} times, got ${actual}`);
    }
  }

  reset(): void {
    this.executions.length = 0;
  }
}
