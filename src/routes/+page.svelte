<script lang="ts">
	import Event from './Event.svelte';
	import CollapsedCard from './CollapsedCard.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { open } from '@tauri-apps/plugin-dialog';

	let showCollapsed = $state(true);
	let events = $state([]);
	let selectedEvent = $state(null);
	let files = $state([]);
	let activeFile = $state(null);
	let parseFileMsg = $state('');

	async function handleFiles(newFiles) {
		let validAdded = false;
		for (let file of newFiles) {
			const ext = file.name.split('.').pop().toLowerCase();
			files = [...files, file];
			validAdded = true;
		}
		if (validAdded && !activeFile) {
			activeFile = files[0];
			await runAutomaticParsing(activeFile);
		}
	}

	async function runAutomaticParsing(file) {
		try {
			const answer = await invoke('parse_file', { path: file.path });
			events = answer;
		} catch (err) {
			console.error('Fehler beim automatischen Parsen:', err);
		}
	}

	async function chooseFile() {
		const selected = await open({
			multiple: false,
			filters: [{ name: 'Daten', extensions: ['xml', 'pts'] }]
		});

		if (selected) {
			const file = {
				name: selected.split(/[/\\]/).pop(),
				path: selected
			};
			handleFiles([file]);
		}
	}
	let groupedEvents = $derived.by(() => {
		if (!showCollapsed) {
			return events.map((event) => ({
				type: event.hasError ? 'error' : 'ok-flat',
				data: event
			}));
		}

		const result = [];
		let currentOkGroup = null;

		for (const event of events) {
			if (event.hasError) {
				if (currentOkGroup) {
					result.push(currentOkGroup);
					currentOkGroup = null;
				}
				result.push({ type: 'error', data: event });
			} else {
				if (!currentOkGroup) {
					currentOkGroup = {
						type: 'collapsed-ok',
						count: 1,
						startTime: event.startTime,
						endTime: event.endTime,
						events: [event]
					};
				} else {
					currentOkGroup.count++;
					currentOkGroup.endTime = event.endTime;
					currentOkGroup.events.push(event);
				}
			}
		}

		if (currentOkGroup) {
			result.push(currentOkGroup);
		}

		return result;
	});

	onMount(() => {
		const unlisten = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === 'drop') {
				const nativePaths = event.payload.paths;
				const mappedFiles = nativePaths.map((filePath) => ({
					name: filePath.split(/[/\\]/).pop(),
					path: filePath
				}));
				handleFiles(mappedFiles);
			}
		});

		return () => {
			unlisten.then((f) => f());
		};
	});
</script>

<main class="container">
	{#if groupedEvents.length > 0}
		<div class="filter-area">
			<label class="switch-container">
				<input type="checkbox" bind:checked={showCollapsed} />
				<span class="switch-slider"></span>
				<span class="switch-label">Fehlerfreie Blöcke zusammenfassen</span>
			</label>
		</div>
	{/if}

	<div class="event-list">
		{#each groupedEvents as item}
			{#if item.type === 'error'}
				<Event
					event={item.data}
					active={selectedEvent?.eventId === item.data.eventId}
					onSelect={() => (selectedEvent = item.data)}
				/>
			{:else if item.type === 'ok-flat'}
				<Event
					event={item.data}
					active={selectedEvent?.eventId === item.data.eventId}
					onSelect={() => (selectedEvent = item.data)}
				/>
			{:else}
				<CollapsedCard {item} />
			{/if}
		{:else}
			<div class="empty-state">
				<p>Es wurden noch keine XML-Events geladen.</p>
			</div>
		{/each}
	</div>

	<div class="drop-container">
		<div class="dropzone">
			<p>Dateien (.xml, .pts) hierher ziehen</p>
			<span class="or-separator">oder</span>
			<button type="button" onclick={chooseFile}>Datei auswählen</button>
		</div>
	</div>
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

	.container {
		margin: 0;
		padding-top: 10vh;
		display: flex;
		flex-direction: column;
		justify-content: center;
		text-align: center;
	}

	.logo {
		height: 6em;
		padding: 1.5em;
		will-change: filter;
		transition: 0.75s;
	}

	.logo.tauri:hover {
		filter: drop-shadow(0 0 2em #24c8db);
	}

	.row {
		display: flex;
		justify-content: center;
	}

	a {
		font-weight: 500;
		color: #646cff;
		text-decoration: inherit;
	}

	a:hover {
		color: #535bf2;
	}

	h1 {
		text-align: center;
	}

	input,
	button {
		border-radius: 8px;
		border: 1px solid transparent;
		padding: 0.6em 1.2em;
		font-size: 1em;
		font-weight: 500;
		font-family: inherit;
		color: #0f0f0f;
		background-color: #ffffff;
		transition: border-color 0.25s;
		box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
	}

	button {
		cursor: pointer;
	}

	button:hover {
		border-color: #396cd8;
	}
	button:active {
		border-color: #396cd8;
		background-color: #e8e8e8;
	}

	.btn-process {
		background-color: #396cd8;
		color: white;
		margin: 10px auto;
		display: block;
		max-width: 200px;
	}

	.loaded-file-info {
		margin: 10px 0;
		font-size: 0.95em;
	}

	input,
	button {
		outline: none;
	}

	@media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}

		a:hover {
			color: #24c8db;
		}

		input,
		button {
			color: #ffffff;
			background-color: #0f0f0f98;
		}
		button:active {
			background-color: #0f0f0f69;
		}
	}

	.drop-container {
		display: flex;
		justify-content: center;
		margin: 20px 0;
	}

	.dropzone {
		width: 80%;
		max-width: 500px;
		padding: 30px;
		border: 2px dashed #396cd8;
		border-radius: 12px;
		background-color: #ffffff;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
		transition:
			background-color 0.2s,
			border-color 0.2s;
	}

	.or-separator {
		font-size: 0.9em;
		color: #666;
		font-style: italic;
	}

	.filter-area {
		display: flex;
		justify-content: center;
		padding: 10px 0;
		margin-bottom: 15px;
	}

	.switch-container {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		gap: 10px;
		user-select: none;
		font-size: 14px;
		color: #475569;
	}

	.switch-container input {
		display: none;
	}

	.switch-slider {
		position: relative;
		width: 40px;
		height: 20px;
		background-color: #cbd5e1;
		border-radius: 20px;
		transition: background-color 0.2s;
	}

	.switch-slider::before {
		content: '';
		position: absolute;
		width: 16px;
		height: 16px;
		left: 2px;
		bottom: 2px;
		background-color: white;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
	}

	.switch-container input:checked + .switch-slider {
		background-color: #3b82f6;
	}

	.switch-container input:checked + .switch-slider::before {
		transform: translateX(20px);
	}

	@media (prefers-color-scheme: dark) {
		.dropzone {
			background-color: #1e1e1e;
			border-color: #24c8db;
		}
		.or-separator {
			color: #aaa;
		}
		.switch-container {
			color: #94a3b8;
		}
		.switch-slider {
			background-color: #475569;
		}
		.switch-container input:checked + .switch-slider {
			background-color: #24c8db;
		}
		.collapsed-card {
			background: #1e1e1e;
			border-color: #334155;
			color: #94a3b8;
		}
		.ok-badge {
			background: #064e3b;
			color: #34d399;
		}
	}
</style>
