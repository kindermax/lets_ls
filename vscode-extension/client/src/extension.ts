/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import { execSync } from "child_process";
import { ExtensionContext } from "vscode";
import * as vscode from "vscode";
import {
  OutputChannel,
} from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  RevealOutputChannelOn,
} from "vscode-languageclient/node";

const SKIP_VERSION_STATE_KEY = "skipUpdate";
const REPO = "https://github.com/kindermax/lets_ls"
let client: LanguageClient;

export function activate(context: ExtensionContext) {
  if (client?.isRunning()) {
    return;
  }

  const outputChannel: OutputChannel = vscode.window.createOutputChannel("Lets LS");

  const config = vscode.workspace.getConfiguration("letsLs");

  const serverOptions: ServerOptions = {
    run: { command: config.get("executablePath") },
    debug: {
      command: config.get("executablePath"),
    },
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      { scheme: "file", language: "yaml", pattern: "**/lets.yaml" },
      { scheme: "file", language: "yaml", pattern: "**/lets.*.yaml" },
    ],
    initializationOptions: {
      log_path: config.get("logPath"),
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

  // Create the language client and start the client.
  client = new LanguageClient(
    "letsLs",
    serverOptions,
    clientOptions,
  );

  // Start the client. This will also launch the server
  client.start();

  context.subscriptions.push(
    vscode.commands.registerCommand('lets-ls.restart', async () => {
      try {
        outputChannel.appendLine('Stopping Lets Language server');
        await client.stop();

        outputChannel.appendLine('Restarting Lets Language server');
        await client.start();
        outputChannel.appendLine('Lets Language server restarted');
      } catch (e) {
        outputChannel.appendLine(`Failed to restart Lets Language server: ${e}`);
      }
    })
  );

  checkUpdates(context, config.get("executablePath"));
}

async function checkUpdates(
  context: ExtensionContext,
  executable: string,
): Promise<void> {
  const res = await fetch(
    `${REPO}/releases/latest`,
  );

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

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}