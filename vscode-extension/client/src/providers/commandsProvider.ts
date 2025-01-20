import * as vscode from 'vscode';
import * as components from '../components';
import * as models from '../models';
import { log } from '../log';

export class CommandsProvider implements vscode.TreeDataProvider<components.TreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<components.CommandTreeItem | undefined> = new vscode.EventEmitter<components.CommandTreeItem | undefined>();
    readonly onDidChangeTreeData: vscode.Event<components.CommandTreeItem | undefined> = this._onDidChangeTreeData.event;

	private commands: models.Command[] = [];
    private treeViewMap: models.CommandsMapping = {};
	private commandNames: string[] = [];

    constructor(
        private nestingEnabled: boolean = false
    ) { }

    setTreeNesting(enabled: boolean) {
        this.nestingEnabled = enabled;
    }

    getTreeItem(element: components.TreeItem): vscode.TreeItem {
        return element;
    }

	getChildren(parent?: components.TreeItem): vscode.ProviderResult<components.TreeItem[]> {
        let treeItems: components.TreeItem[] = [];
        let taskTreeItems: components.CommandTreeItem[] = [];

        this.commandNames.forEach(commandName => {
            if (commandName in this.treeViewMap) {
                let item = new components.CommandTreeItem(
					commandName,
                    this.treeViewMap[commandName],
                    vscode.TreeItemCollapsibleState.None,
					undefined,  // TODO: add go to definition and run
                );
				taskTreeItems.push(item);
            }
        });

        treeItems = treeItems.concat(taskTreeItems);

        return Promise.resolve(treeItems);
	}

    refresh(commands?: models.Command[]) {
		if (commands) {
			this.commands = commands;
			this.treeViewMap = {};
			this.commandNames = [];
			this.commands.forEach((command) => {
				this.treeViewMap[command.name] = command;
				this.commandNames.push(command.name);
			});
            this.commandNames.sort((a, b) => (a > b ? -1 : 1));
		}
        this._onDidChangeTreeData.fire(undefined);
    }

}