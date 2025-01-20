import * as vscode from 'vscode';
import * as models from '../models';

export type TreeItem = WorkspaceTreeItem | CommandTreeItem;

export class WorkspaceTreeItem extends vscode.TreeItem {
    constructor(
        readonly label: string,
        readonly workspace: string,
        readonly letsCommands: models.Command[],
        readonly collapsibleState: vscode.TreeItemCollapsibleState,
        readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.description = this.workspace;
        this.iconPath = new vscode.ThemeIcon('folder', new vscode.ThemeColor('letsls.workspaceIcon'));
        this.contextValue = `workspaceTreeItem`;
    }
}

export class CommandTreeItem extends vscode.TreeItem {
    constructor(
        readonly label: string,
        readonly letsCommand: models.Command,
        readonly collapsibleState: vscode.TreeItemCollapsibleState,
        readonly command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.description = this.letsCommand?.description || "";
        if (this.description.startsWith("No description")) {
            this.description = false;
        };
        this.iconPath = new vscode.ThemeIcon('debug-breakpoint-log-unverified', new vscode.ThemeColor('letsls.upToDateIcon'));
        this.contextValue = `commandTreeItem`;
    }
}