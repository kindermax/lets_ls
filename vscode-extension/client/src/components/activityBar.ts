import * as vscode from 'vscode';
import * as providers from '../providers';
import * as models from '../models';

export class ActivityBar {
    private _provider: providers.CommandsProvider;

    constructor() {
        this._provider = new providers.CommandsProvider();
        vscode.window.createTreeView('lets-ls.commands', {
            treeDataProvider: this._provider,
            showCollapseAll: true
        });
    }

    public setTreeNesting(enabled: boolean) {
        this._provider.setTreeNesting(enabled);
        this._provider.refresh();
    }

    public refresh(commands?: models.Command[]) {
        this._provider.refresh(commands);
    }
}