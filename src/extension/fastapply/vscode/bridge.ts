/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import * as vscode from 'vscode';

const RUST_API_URL = 'http://127.0.0.1:3000';

export interface RustContext {
	found: boolean;
	start_line: number;
	end_line: number;
	message: string;
}

export class FastApplyBridge {
	public static async findComponent(document: vscode.TextDocument, query: string): Promise<RustContext | null> {
		try {
			const match = query.match(/\b[A-Z][a-zA-Z0-9]*\b/);
			const targetName = match ? match[0] : null;

			if (!targetName) {
				return null;
			}

			// Use global fetch directly
			const response = await fetch(`${RUST_API_URL}/find-component`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source_code: document.getText(),
					target_name: targetName
				})
			});

			return await response.json() as RustContext;
		} catch (e) {
			console.error("FastApply Connection Failed:", e);
			return null;
		}
	}

	public static async selectContext(editor: vscode.TextEditor, context: RustContext) {
		if (!context.found) {
			return;
		}
		const range = new vscode.Range(context.start_line, 0, context.end_line, 0);
		editor.revealRange(range, vscode.TextEditorRevealType.InCenter);
		editor.selection = new vscode.Selection(range.start, range.end);
	}
}