import { execSync } from "child_process";
import * as vscode from "vscode";
import { ExtensionContext, OutputChannel } from "vscode";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    RevealOutputChannelOn,
    Executable,
} from "vscode-languageclient/node";

import * as components from "./components";
import * as services from "./services";
import * as models from "./models";
import { log } from './log';

const SKIP_VERSION_STATE_KEY = "skipUpdate";
const REPO = "https://github.com/kindermax/lets_ls"


export class LetsExtension {
    public client: LanguageClient;

    private _activityBar: components.ActivityBar;
    private letsService: services.LetsService
    private letsState: models.LetsState

    constructor() {
        this._activityBar = new components.ActivityBar();
        this.letsService = new services.LetsService();
        this.letsState = new models.LetsState();
    }

    isRunning() {
        return this.client?.isRunning();
    }

    activate(context: ExtensionContext) {
        const outputChannel: OutputChannel = vscode.window.createOutputChannel("Lets LS");

        const config = vscode.workspace.getConfiguration("letsLs");
        const executablePath: string = config.get("executablePath");
        const debug: boolean = config.get("debug");
        const logPath: string = config.get("logPath");

        let env = null;
        if (debug) {
            env = {
                RUST_LOG: "debug",
            };
        }
        let run: Executable = {
            command: executablePath,
            options: {
                env
            }
        };
        const serverOptions: ServerOptions = {
            run,
            debug: run,
        };

        const clientOptions: LanguageClientOptions = {
            documentSelector: [
                { scheme: "file", language: "yaml", pattern: "**/lets.yaml" },
                { scheme: "file", language: "yaml", pattern: "**/lets.*.yaml" },
            ],
            initializationOptions: {
                log_path: logPath,
            },
            outputChannel,
            outputChannelName: 'Lets Language Server',
            revealOutputChannelOn: RevealOutputChannelOn.Never,
            initializationFailedHandler(err) {
                outputChannel.appendLine('Initialization failed');
                outputChannel.appendLine(err.message);
                if (err.stack) {
                    outputChannel.appendLine(err.stack);
                }
                return false;
            },
        };

        this.client = new LanguageClient(
            "letsLs",
            serverOptions,
            clientOptions,
        );

        // Start the client. This will also launch the server
        this.client.start();

        this.registerCommands(context, outputChannel);
        // this.checkUpdates(context, executablePath);
    }
    deactivate(): Promise<void> {
        return this.client.stop()
    }

    setTreeNesting(enabled: boolean): void {
        this._activityBar.setTreeNesting(enabled);
        vscode.commands.executeCommand('setContext', 'lets-ls:treeNesting', enabled);
    }

    async refresh() {
        this.letsState.commands = await this.letsService.readCommands();
        this._activityBar.refresh(this.letsState.commands);
    }

    registerCommands(context: ExtensionContext, outputChannel: OutputChannel) {
        context.subscriptions.push(
            vscode.commands.registerCommand('lets-ls.restart', async () => {
                try {
                    outputChannel.appendLine('Stopping Lets Language server');
                    await this.client.stop();

                    outputChannel.appendLine('Restarting Lets Language server');
                    await this.client.start();
                    outputChannel.appendLine('Lets Language server restarted');
                } catch (e) {
                    outputChannel.appendLine(`Failed to restart Lets Language server: ${e}`);
                }
            })
        );
        // Refresh commands
        context.subscriptions.push(vscode.commands.registerCommand('lets-ls.refresh', () => {
            log.info("Command: lets-ls.refresh");
            this.refresh();
        }));

        // View commands as list
        context.subscriptions.push(vscode.commands.registerCommand('lets-ls.showCommands', () => {
            log.info("Command: lets-ls.showCommands");
            this.setTreeNesting(false);
        }));

        context.subscriptions.push(vscode.commands.registerCommand('lets-ls.runCommand', (treeItem?: components.CommandTreeItem) => {
            log.info("Command: lets-ls.runCommand");
            if (treeItem?.letsCommand) {
                this.letsService.runCommand(treeItem.letsCommand);
            }
        }));
    }

    async checkUpdates(context: ExtensionContext, executable: string): Promise<void> {
        const res = await fetch(`${REPO}/releases/latest`);

        // js is perfect
        const { tag_name } = (await res.json()) as any;

        //check if skipped
        const val = context.globalState.get(SKIP_VERSION_STATE_KEY);
        if (val && val === tag_name) {
            return;
        }

        const version = execSync(`${executable} --version`).toString();

        // older version which doesn't support --version
        if (!version) {
            return;
        }

        // format of: lets_ls X.X.X
        const versionSplit = version.split(" ");

        // shouldn't occur
        if (versionSplit.length != 2) {
            return;
        }

        const versionTag = versionSplit[1].trim();

        if (tag_name != versionTag) {
            vscode.window
                .showInformationMessage(
                    "There is a newer version of Lets language server.",
                    "Show installation guide",
                    "Show changes",
                    "Skip this version",
                )
                .then((answer) => {
                    let url = "";
                    if (answer === "Show changes") {
                        url = `${REPO}/compare/${versionTag}...${tag_name}`;
                    } else if (answer === "Show installation guide") {
                        url =
                            `${REPO}?tab=readme-ov-file#installation`;
                    } else if (answer === "Skip this version") {
                        context.globalState.update(SKIP_VERSION_STATE_KEY, tag_name);
                    }

                    if (url != "") {
                        vscode.env.openExternal(vscode.Uri.parse(url));
                    }
                });
        }
    }

}