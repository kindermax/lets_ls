import * as cp from 'child_process';
import * as vscode from 'vscode';

import * as models from '../models';
import { log } from '../log';


type ExecutionResult = {
	stdout: string;
	stderr: string;
	error: cp.ExecException | null;
	hasError: boolean;
};

export class LetsService {
	private async execute(command: string, dir?: string): Promise<ExecutionResult> {
        return await new Promise((resolve) => {
			cp.exec(command, { cwd: dir }, (error: cp.ExecException | null, stdout: string, stderr: string) => {
				return resolve({ stdout, stderr, error, hasError: !!error || stderr.length > 0 });
			});
		});
	}

    async runCommand(letsCommand: models.Command) {
		log.info(`[TODO] Running command: ${letsCommand.name}`);
    }

    async readCommands(): Promise<models.Command[]> {
		const dir = vscode.workspace.workspaceFolders?.[0].uri.fsPath;
		const result = await this.execute("lets completion --list --verbose", dir);
		if (result.hasError) {
			return [];
		}

		const lines = result.stdout.trim().split("\n");
		return lines
		.map(line => {
			const [name, description] = line.split(":");
			return new models.Command(name, description);
		});
    }
}