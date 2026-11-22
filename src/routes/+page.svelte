<script lang="ts">
	import type { PtsFile } from './PtsFile.ts';
	import type { ConfigFile } from './ConfigFile.ts';
	import type { Param } from './Param.ts';
	import { listen } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { open } from '@tauri-apps/plugin-dialog';
	import FileView from './FileView.svelte';
	import Sidebar from './Sidebar.svelte';
	import HeaderBar from './HeaderBar.svelte';
	import StatusBar from './StatusBar.svelte';
	import ConfigPage from './ConfigPage.svelte';
	import DatabaseView from './DatabaseView.svelte';

	let showConfigPage = $state(false);
	let showDbPage = $state(false);
	let showCollapsed = $state(true);
	let showVaEvents = $state(false);
	let theme = $state({ value: false, name: 'light' });
	let files = $state<PtsFile[]>([]);
	let activeFile = $state<PtsFile | null>(null);
	let configFile = $state<ConfigFile>({
		name: 'config-file',
		path: '.env'
	});
	let configParams = $state<Param[] | null>(null);

	let dbStatus = $state<'unknown' | 'connected' | 'error'>('unknown');
	let syncStatus = $state<'idle' | 'syncing' | 'synced' | 'error'>('idle');
	let syncMessage = $state<string>('');

	function getParamValue(params: Param[] | null, key: string, fallback = ''): string {
		return params?.find((p) => p.name === key)?.value ?? fallback;
	}

	async function connectToDb(params: Param[]) {
		try {
			const db = getParamValue(params, 'DATABASE_URL', 'ses2.db');
			const count = await invoke<number>('check_db_connection', { db });
			dbStatus = 'connected';
			syncMessage = `${count} Einträge in DB`;
		} catch (err) {
			dbStatus = 'error';
			syncMessage = String(err);
		}
	}

	function triggerBackgroundSync(params: Param[]) {
		const email = getParamValue(params, 'EMAIL');
		const password = getParamValue(params, 'PASSWD');
		const db = getParamValue(params, 'DATABASE_URL', 'ses2.db');
		const url = getParamValue(params, 'SES_URL', 'https://www.ses-360.com/');
		const headlessRaw = getParamValue(params, 'HEADLESS', 'true').trim().toLowerCase();
		const headless = headlessRaw === '' ? true : ['1', 'true', 'yes'].includes(headlessRaw);

		syncStatus = 'syncing';
		void invoke('start_sync', {
			email,
			password,
			db,
			url,
			headless
		}).catch((err) => {
			syncStatus = 'error';
			syncMessage = String(err);
		});
	}

	function deleteFile(filePath: string) {
		files = files.filter((f) => f.path !== filePath);
		if (activeFile?.path === filePath) {
			activeFile = files.length > 0 ? files[files.length - 1] : null;
		}
	}

	function applyTheme(isDark: boolean) {
		const themeName = isDark ? 'dark' : 'light';
		theme = { value: isDark, name: themeName };
		document.documentElement.setAttribute('data-theme', themeName);
		localStorage.setItem('theme', themeName);
	}

	function toggleTheme() {
		applyTheme(!theme.value);
	}

	function getTheme() {
		return theme.value;
	}

	async function loadConfig(configFile: ConfigFile) {
		try {
			configParams = await invoke<Param[]>('load_or_create_config', { path: configFile.path });
		} catch (err) {
			console.error('Fehler beim laden der config:', err);
		}
	}

	async function handleFiles(newFiles: { name: string; path: string }[]) {
		let validAdded = false;
		for (let file of newFiles) {
			const myfile: PtsFile = {
				name: file.name,
				path: file.path,
				si_events: [],
				va_events: [],
				si_events_errors: 0,
				va_events_errors: 0
			};

			files = [...files, myfile];
			validAdded = true;
		}
		if (validAdded) {
			activeFile = files[files.length - 1];
			await runAutomaticParsing(activeFile);
		}
	}

	async function runAutomaticParsing(file: PtsFile) {
		try {
			const answer: PtsFile = await invoke('parse_file', { path: file.path });
			file.si_events_errors = answer.si_events_errors;
			file.va_events_errors = answer.va_events_errors;
			file.si_events = answer.si_events as any[];
			file.va_events = answer.va_events as any[];
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
		applyTheme(savedTheme === 'dark');

		const unlistenDrop = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === 'drop') {
				const mappedFiles = event.payload.paths.map((filePath) => ({
					name: filePath.split(/[/\\]/).pop() ?? 'unknown filename',
					path: filePath ?? 'unknown path'
				}));
				handleFiles(mappedFiles);
			}
		});

		const unlistenProgress = listen<{ page: number; new_rows: number; known_rows: number }>(
			'sync-progress',
			(event) => {
				syncStatus = 'syncing';
				syncMessage = `Seite ${event.payload.page} (+${event.payload.new_rows} neu)`;
			}
		);

		const unlistenFinished = listen<{ ok: boolean; message: string; total_in_db?: number }>(
			'sync-finished',
			(event) => {
				syncStatus = event.payload.ok ? 'synced' : 'error';
				syncMessage = event.payload.message;
			}
		);

		void (async () => {
			await loadConfig(configFile);
			if (configParams) {
				await connectToDb(configParams);
				triggerBackgroundSync(configParams);
			}
		})();

		return () => {
			unlistenDrop.then((f) => f());
			unlistenProgress.then((f) => f());
			unlistenFinished.then((f) => f());
		};
	});
</script>

<main class="layout">
	<HeaderBar
		{showConfigPage}
		{files}
		{toggleTheme}
		{getTheme}
		{activeFile}
		bind:showVaEvents
		bind:showCollapsed
	/>
	{#if showConfigPage}
		<div class="file-view-container">
			<ConfigPage {configParams} />
		</div>
	{:else if showDbPage}
		<div class="file-view-container">
			<DatabaseView {configParams} />
		</div>
	{:else}
		<div class="content-body {files.length === 0 ? 'empty-state' : ''}">
			<Sidebar
				{files}
				bind:activeFile
				onFileSelect={chooseFile}
				isExpanded={files.length > 0}
				onFileDelete={deleteFile}
			/>
			{#if files.length > 0}
				<div class="file-view-container">
					<FileView {showVaEvents} {showCollapsed} {activeFile} />
				</div>
			{/if}
		</div>
	{/if}
	<StatusBar
		bind:showConfigPage
		bind:showDbPage
		{dbStatus}
		{syncStatus}
		{syncMessage}
		onResync={() => configParams && triggerBackgroundSync(configParams)}
	/>
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

	.layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		margin: 0;
		overflow: hidden;
	}

	.content-body {
		display: flex;
		flex: 1;
		min-height: 0;
		width: 100%;
		overflow: hidden;
	}

	.file-view-container {
		display: flex;
		flex-direction: row;
		flex-grow: 1;
		min-width: 0;
		overflow-x: auto;
	}

	.empty-state {
		justify-content: center;
		align-items: center;
	}

	.file-view-container {
		display: flex;
		flex: 1;
		min-width: 0;
	}

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

		--theme-filter-background: #f6f6f6;
		--theme-filter-border: #0f0f0f;
		--theme-filter-color: #0f0f0f;
		--theme-filter-border-hover: #396cd8;

		--mode-background: #f6f6f6;
		--mode-border: #0f0f0f;
		--mode-color: #0f0f0f;
		--mode-border-hover: #396cd8;

		--mode-inactive-background: #e9e9e9;
		--mode-inactive-border: #cccccc;
		--mode-inactive-color: #999999;
		--mode-inactive-border-hover: #396cd8;

		--trash-color: #0f0f0f;
		--trash-hover-color: #ff2200;

		--bg-color: #f6f6f6;
		--text-color: #0f0f0f;
		--input-bg: #ffffff;
		--input-text: #0f0f0f;
		--link-hover: #535bf2;
		--details-card-color: #000000;

		--event-card-background: #f8fafc;
		--event-card-border: #e2e8f0;
		--event-card-background-hover: #f1f5f9;
		--event-card-border-hover: #cbd5e1;
		--event-card-background-active: #eff6ff;
		--event-card-border-active: #3b82f6;
		--event-card-background-has-error: #fef2f2;
		--event-card-border-has-error: #f87171;
		--event-card-background-has-error-hover: #f87171 !important;
		--event-card-border-has-error: #ed4747 !important;
		--event-card-top: #64748b;
		--event-card-title: #1e293b;
		--event-card-error-badge-background: #fee2e2;
		--event-card-error-badge-color: #dc2626;
		--event-card-bottom: #475569;

		--collapsed-card-background: #ffffff;
		--collapsed-card-border: #e2e8f0;
		--collapsed-card-color: #475569;

		--xml-container-background: #b5b5b3;
		--xml-bracket: black;
		--xml-tag: #910085;
		--xml-attr: #0743b3;
		--xml-string: green;
	}

	:global([data-theme='dark']) {
		--theme-switch-background: #2f2f2f;
		--theme-switch-border: #f6f6f6;
		--theme-switch-color: #f6f6f6;
		--theme-switch-border-hover: #396cd8;

		--theme-filter-background: #2f2f2f;
		--theme-filter-border: #f6f6f6;
		--theme-filter-color: #f6f6f6;
		--theme-filter-border-hover: #396cd8;

		--mode-background: #2f2f2f;
		--mode-border: #f6f6f6;
		--mode-color: #f6f6f6;
		--mode-border-hover: #396cd8;

		--mode-inactive-background: #1e1e1e;
		--mode-inactive-border: #444444;
		--mode-inactive-color: #777777;
		--mode-inactive-border-hover: #396cd8;

		--trash-color: #f6f6f6;
		--trash-hover-color: #ff2200;

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
		--event-card-background-has-error-hover: #5c1d1d !important;
		--event-card-border-has-error-hover: #991b1b !important;
		--event-card-top: #94a3b8;
		--event-card-title: #f8fafc;
		--event-card-error-badge-background: #7f1d1d;
		--event-card-error-badge-color: #fca5a5;
		--event-card-bottom: #cbd5e1;

		--collapsed-card-background: #1e293b;
		--collapsed-card-border: #334155;
		--collapsed-card-color: #f8fafc;

		--xml-container-background: #616161;
		--xml-bracket: #94a3b8;
		--xml-tag: #f43f5e;
		--xml-attr: #38bdf8;
		--xml-string: #facc15;
	}

	:global(body) {
		background-color: var(--bg-color);
		color: var(--text-color);
		transition:
			background-color 0.3s,
			color 0.3s;
	}
</style>
