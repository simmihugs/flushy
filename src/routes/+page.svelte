<script lang="ts">
	import type { MyFile } from './MyFile.ts';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { open } from '@tauri-apps/plugin-dialog';
	import FileView from './FileView.svelte';
	import Sidebar from './Sidebar.svelte';

	let showCollapsed = $state(true);
	let theme = $state({ value: false, name: 'light' });
	let files = $state<MyFile[]>([]);
	let activeFile = $state<MyFile | null>(null);

	function applyTheme(isDark: boolean) {
		const themeName = isDark ? 'dark' : 'light';
		theme = { value: isDark, name: themeName };
		document.documentElement.setAttribute('data-theme', themeName);
		localStorage.setItem('theme', themeName);
		console.log(theme);
	}

	function toggleTheme() {
		applyTheme(!theme.value);
	}

	function getTheme() {
		return theme.value ? '🌙 Dark' : '☀️ Light';
	}

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
		const savedTheme = localStorage.getItem('theme');
		const isDark = savedTheme === 'dark';

		applyTheme(isDark);

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

<main class="layout {files.length === 0 ? 'empty-state' : 'active-state'}">
	<button class="theme-switch" onclick={toggleTheme}>{getTheme()}</button>
	<Sidebar {files} bind:activeFile onFileSelect={chooseFile} isExpanded={files.length > 0} />
	{#if files.length > 0}
		<div class="file-view-container">
			<FileView {showCollapsed} {activeFile} />
		</div>
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

	:global(body) {
		margin: 0;
		padding: 0;
		overflow: hidden;
	}

	/* .theme-switch { */
	/* 	position: absolute; */
	/* 	top: 15px; */
	/* 	right: 15px; */
	/* 	z-index: 1000; */
	/* 	cursor: pointer; */
	/* 	padding: 8px 12px; */
	/* 	border-radius: 8px; */

	/* 	cursor: pointer; */
	/* 	background: var(--theme-switch-background); */
	/* 	color: var(--theme-switch-color); */
	/* 	border: 1px solid transparent; */
	/* 	transition: var(--theme-switch-border) 0.25s; */
	/* 	box-shadow: 0 1px 1px var(--theme-switch-border); */
	/* } */

	/* .theme-switch:hover { */
	/* 	border: var(--theme-switch-border-hover); */
	/* } */
	.theme-switch {
		position: absolute;
		top: 15px;
		right: 15px;
		z-index: 1000;
		cursor: pointer;
		padding: 8px 12px;
		border-radius: 8px;

		background: var(--theme-switch-background);
		color: var(--theme-switch-color);

		/* Festen Rahmen definieren und nur die border-color animieren: */
		border: 1px solid var(--theme-switch-border);
		transition:
			border-color 0.25s ease,
			background-color 0.25s ease;

		box-shadow: 0 1px 1px rgba(0, 0, 0, 0.1);
	}

	.theme-switch:hover {
		border-color: var(--theme-switch-border-hover);
	}

	.layout {
		display: flex;
		height: 100vh;
		width: 100vw;
		margin: 0;
		overflow: hidden;
	}

	/* .sidebar { */
	/* 	flex: 0 0 300px; */
	/* } */

	.empty-state {
		justify-content: center;
		align-items: center;
	}

	/* .sidebar.centered { */
	/* 	width: 100%; */
	/* 	display: flex; */
	/* 	justify-content: center; */
	/* 	align-items: center; */
	/* } */

	.active-state {
		flex-direction: row;
	}

	.file-view-container {
		display: flex;
		flex: 1;
		min-width: 0;
	}

	/* .cool { */
	/* 	border: 2px solid red; */
	/* 	flex: 1; */
	/* 	min-width: 0; */
	/* 	display: flex; */
	/* } */

	/* .sidebar.centered { */
	/* 	width: 100%; */
	/* 	display: flex; */
	/* 	justify-content: center; */
	/* 	align-items: center; */
	/* } */

	/* .sidebar.expanded { */
	/* 	width: 300px; */
	/* 	flex-shrink: 0; */
	/* 	border-right: 1px solid #ccc; */
	/* 	display: flex; */
	/* 	flex-direction: column; */
	/* 	justify-content: space-between; */
	/* } */

	.file-view-container {
		display: flex;
		flex-direction: row;
		flex-grow: 1;
		overflow-x: auto;
	}

	:root,
	:global([data-theme='light']) {
		--theme-switch-background: #f6f6f6;
		--theme-switch-border: #0f0f0f;
		--theme-switch-color: #0f0f0f;
		--theme-switch-border-hover: #396cd8;

		--bg-color: #f6f6f6;
		--text-color: #0f0f0f;
		--input-bg: #ffffff;
		--input-text: #0f0f0f;
		--link-hover: #535bf2;
		--details-card-color: #000000;

		/* event-card */
		--event-card-background: #f8fafc;
		--event-card-border: #e2e8f0;
		--event-card-background-hover: #f1f5f9;
		--event-card-border-hover: #cbd5e1;
		--event-card-background-active: #eff6ff;
		--event-card-border-active: #3b82f6;
		--event-card-background-has-error: #fef2f2;
		--event-card-border-has-error: : #f87171;
		--event-card-background-has-error-hover: #f87171 !important;
		--event-card-border-has-error: #ed4747 !important;
		--event-card-top: #64748b;
		--event-card-title: #1e293b;
		--event-card-error-badge-background: #fee2e2;
		--event-card-error-badge-color: #dc2626;
		--event-card-bottom: #475569;

		/* collapsed cards */
		--collapsed-card-background: #ffffff;
		--collapsed-card-border: #e2e8f0;
		--collapsed-card-color: #475569;
	}

	:global([data-theme='dark']) {
		--theme-switch-background: #2f2f2f;
		--theme-switch-border: #f6f6f6;
		--theme-switch-color: #f6f6f6;
		--theme-switch-border-hover: #396cd8;

		--bg-color: #2f2f2f;
		--text-color: #f6f6f6;
		--input-bg: #1e1e1e;
		--input-text: #ffffff;
		--link-hover: #24c8db;
		--details-card-color: #e2e8f0;

		--event-card-background: #1e293b;
		--event-card-border: #334155;
		--event-card-background-hover: #334155;
		--event-card-border-hover: #475569;
		--event-card-background-active: #1e3a8a;
		--event-card-border-active: #3b82f6;
		--event-card-background-has-error: #451a1a;
		--event-card-border-has-error: #7f1d1d;
		--event-card-background-has-error-hover: #5c1d1d;
		--event-card-border-has-error-hover: #991b1b;
		--event-card-top: #94a3b8;
		--event-card-title: #f8fafc;
		--event-card-error-badge-background: #7f1d1d;
		--event-card-error-badge-color: #fca5a5;
		--event-card-card-bottom: #cbd5e1;

		--collapsed-card-background: #1e293b;
		--collapsed-card-border: #334155;
		--collapsed-card-color: #f8fafc;
	}

	:global(body) {
		background-color: var(--bg-color);
		color: var(--text-color);
		transition:
			background-color 0.3s,
			color 0.3s;
	}

	/* input, */
	/* button { */
	/* 	color: var(--input-text); */
	/* 	background-color: var(--input-bg); */
	/* } */

	/* a:hover { */
	/* 	color: var(--link-hover); */
	/* } */
</style>
