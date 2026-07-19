<script lang="ts">
	import type { MyFile } from './MyFile.ts';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { open } from '@tauri-apps/plugin-dialog';
	import FileView from './FileView.svelte';
	import Sidebar from './Sidebar.svelte';

	let showCollapsed = $state(true);
	let selectedEvent = $state(null);
	let files = $state<MyFile[]>([]);
	let activeFile = $state<MyFile | null>(null);

	async function handleFiles(newFiles: { name: string; path: string }[]) {
		let validAdded = false;
		for (let file of newFiles) {
			const myfile: MyFile = {
				name: file.name,
				path: file.path,
				events: []
			};

			files = [...files, myfile];
			validAdded = true;
		}
		if (validAdded) {
			activeFile = files[files.length - 1];
			await runAutomaticParsing(activeFile);
		}
	}

	async function runAutomaticParsing(file: MyFile) {
		try {
			const answer = await invoke('parse_file', { path: file.path });
			file.events = answer as any[];
		} catch (err) {
			console.error('Fehler beim automatischen Parsen:', err);
		}
	}

	async function chooseFile() {
		const selected = await open({
			multiple: false,
			filters: [{ name: 'Daten', extensions: ['xml', 'pts'] }]
		});

		if (typeof selected === 'string') {
			const file = {
				name: selected.split(/[/\\]/).pop() ?? 'unknownfile',
				path: selected
			};
			handleFiles([file]);
		}
	}

	onMount(() => {
		const unlisten = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === 'drop') {
				const nativePaths = event.payload.paths;
				const mappedFiles = nativePaths.map((filePath) => ({
					name: filePath.split(/[/\\]/).pop() ?? 'unknown filename',
					path: filePath ?? 'unknown path'
				}));
				handleFiles(mappedFiles);
			}
		});

		return () => {
			unlisten.then((f) => f());
		};
	});
</script>

<!-- <main class="container"> -->
<!-- 	<Sidebar {files} bind:activeFile onFileSelect={chooseFile} /> -->
<!-- 	<FileView {showCollapsed} {activeFile} {selectedEvent} /> -->
<!-- </main> -->

<main class="layout {files.length === 0 ? 'empty-state' : 'active-state'}">
	<Sidebar {files} bind:activeFile onFileSelect={chooseFile} isExpanded={files.length > 0} />

	{#if files.length > 0}
		<section class="file-view-container">
			<FileView {showCollapsed} {activeFile} {selectedEvent} />
		</section>
	{/if}
</main>

<style>
	:root {
		font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
		font-size: 16px;
		line-height: 24px;
		font-weight: 400;

		color: #0f0f0f;
		background-color: #f6f6f6;

		font-synthesis: none;
		text-rendering: optimizeLegibility;
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
		-webkit-text-size-adjust: 100%;
	}

	/* .container { */
	/* 	margin: 0; */
	/* 	padding-top: 10vh; */
	/* 	display: flex; */
	/* 	flex-direction: column; */
	/* 	justify-content: center; */
	/* 	text-align: center; */
	/* } */

	@media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}
	}

	:global(body) {
		margin: 0;
		padding: 0;
		overflow: hidden;
	}

	.layout {
		display: flex;
		height: 100vh;
		min-width: 100vw;
		margin: 0;
	}

	.sidebar {
		flex-shrink: 0;
		width: 300px;
	}

	.empty-state {
		justify-content: center;
		align-items: center;
	}

	.sidebar.centered {
		width: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}
	.active-state {
		flex-direction: row;
	}

	.file-view-container {
		flex: 1;
		overflow-y: auto;
	}

	.sidebar.centered {
		width: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.sidebar.expanded {
		width: 300px;
		flex-shrink: 0;
		border-right: 1px solid #ccc;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
	}

	.file-view-container {
		display: flex;
		flex-direction: row;
		flex-grow: 1;
		overflow-x: auto;
	}
</style>
