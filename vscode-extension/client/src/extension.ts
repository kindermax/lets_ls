/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */
import { ExtensionContext } from "vscode";
import { LetsExtension } from "./lets";

let extension: LetsExtension;

export function activate(context: ExtensionContext) {
  if (extension?.isRunning()) {
    return;
  }

  extension = new LetsExtension();
  extension.activate(context);
  extension.refresh();
}

export function deactivate(): Thenable<void> | undefined {
  if (!extension) {
    return undefined;
  }
  return extension.deactivate();
}