<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	let files = [];
	let activeFile = null;
	let isSidebarCollapsed = false;
	let events = [];
	let errorMessage = '';
	let selectedEvent = null;

	const ALLOWED = ['xml', 'pts'];

	// Nimmt Objekte der Form { name, path } entgegen
	function handleFiles(newFiles) {
		let validAdded = false;
		for (let file of newFiles) {
			const ext = file.name.split('.').pop().toLowerCase();
			if (ALLOWED.includes(ext)) {
				files = [...files, file];
				validAdded = true;
			} else {
				alert(`Fehler: .${ext.toUpperCase()} ist nicht erlaubt!`);
			}
		}
		if (validAdded && !activeFile) {
			selectFile(files[files.length - 1]);
		}
	}

	async function selectFile(file) {
		activeFile = file;
		errorMessage = '';
		events = [];
		console.log(file);

		if (!file.path) {
			errorMessage = 'Kritischer Fehler: Kein Pfad vorhanden!';
			return;
		}

		try {
			const response = await invoke('process_file_to_json', {
				path: file.path
			});
			events = response;
		} catch (err) {
			errorMessage = `Rust-Fehler: ${err}`;
		}
	}

	async function chooseFile() {
		try {
			const selected = await open({
				multiple: true,
				filters: [{ name: 'Daten', extensions: ALLOWED }]
			});

			if (selected) {
				// Falls mehrere Dateien ausgewählt wurden (selected ist dann ein Array von Pfaden)
				const paths = Array.isArray(selected) ? selected : [selected];
				const mapped = paths.map((filePath) => ({
					name: filePath.split(/[/\\]/).pop(),
					path: filePath
				}));
				handleFiles(mapped);
			}
		} catch (err) {
			errorMessage = `Fehler beim Öffnen des Dialogs: ${err}`;
		}
	}

	onMount(() => {
		const unlisten = getCurrentWebview().onDragDropEvent((event) => {
			if (event.payload.type === 'drop') {
				const mapped = event.payload.paths.map((filePath) => ({
					name: filePath.split(/[/\\]/).pop(),
					path: filePath
				}));
				handleFiles(mapped);
			}
		});

		return () => {
			unlisten.then((f) => f());
		};
	});
</script>

<div class="dropzone">
	{#if files.length === 0}
		<div transition:fade>
			<p>Dateien hierher in das Fenster ziehen oder</p>
			<button class="btn" on:click={chooseFile}>Datei wählen</button>
		</div>
	{:else}
		<div transition:fade style="width: 100%;">
			<button class="btn-mini" on:click={chooseFile}>
				{isSidebarCollapsed ? '＋' : '＋ Datei hinzufügen'}
			</button>
		</div>
	{/if}
</div>

<svelte:window on:dragover|preventDefault on:drop|preventDefault />

<div class="app-container" class:has-files={files.length > 0} class:collapsed={isSidebarCollapsed}>
	<aside class="sidebar-area">
		{#if files.length > 0}
			<div class="sidebar-header" transition:fade>
				<button on:click={() => (isSidebarCollapsed = !isSidebarCollapsed)}>
					{isSidebarCollapsed ? '➔' : '←'}
				</button>
				{#if !isSidebarCollapsed}
					<span class="active-title">Aktiv: {activeFile?.name}</span>
				{/if}
			</div>

			<div class="file-list">
				{#each files as file}
					<button
						class="file-item"
						class:active={activeFile === file}
						on:click={() => (activeFile = file)}
					>
						<span class="icon">📄</span>
						{#if !isSidebarCollapsed}
							<span class="text" transition:fade>{file.name}</span>
						{/if}
					</button>
				{/each}
			</div>
		{/if}

		<div
			class="dropzone"
			on:dragover|preventDefault
			on:drop|preventDefault|stopPropagation={handleDrop}
		>
			{#if files.length === 0}
				<div transition:fade>
					<p>Dateien hierher ziehen oder</p>
					<input
						type="file"
						id="file-input"
						multiple
						on:change={(e) => handleFiles(Array.from(e.target.files))}
						hidden
					/>
					<label for="file-input" class="btn">Datei wählen</label>
				</div>
			{:else}
				<div transition:fade style="width: 100%;">
					<input
						type="file"
						id="file-input-mini"
						multiple
						on:change={(e) => handleFiles(Array.from(e.target.files))}
						hidden
					/>
					<label for="file-input-mini" class="btn-mini">
						{isSidebarCollapsed ? '＋' : '＋ Datei hinzufügen'}
					</label>
				</div>
			{/if}
		</div>
	</aside>

	<main class="main-content">
		{#if files.length === 0}
			<div class="empty-state">Bitte lade eine Datei hoch, um zu beginnen.</div>
		{:else}
			<div class="viewer" transition:fade>
				<div class="viewer-header">
					<h2>Inhalt von: {activeFile?.name}</h2>
					<p>Größe: {((activeFile?.size || 0) / 1024).toFixed(2)} KB</p>
				</div>

				{#if errorMessage}
					<div class="error-banner">{errorMessage}</div>
				{/if}

				<div class="event-grid">
					{#each events as event}
						<button
							class="event-card"
							class:has-error={event.hasError}
							class:active={selectedEvent?.eventId === event.eventId}
							on:click={() => (selectedEvent = event)}
						>
							<div class="card-top">
								<span class="event-id">{event.eventId}</span>
								{#if event.hasError}
									<span class="error-badge">
										{event.errorFront || event.errorBack || '⚠️ Fehler'}
									</span>
								{/if}
							</div>

							<h3 class="event-title">{event.title}</h3>

							<div class="card-bottom">
								<span class="time">{formatTime(event.startTime)} - {formatTime(event.endTime)}</span
								>
							</div>
						</button>
					{/each}
				</div>

				{#if selectedEvent}
					<div class="detail-panel" transition:fade>
						<h3>Detail: {selectedEvent.title}</h3>
						<p>
							<strong>Start:</strong>
							{new Date(selectedEvent.startTime).toLocaleString('de-DE')}
						</p>
						<p>
							<strong>Ende:</strong>
							{new Date(selectedEvent.endTime).toLocaleString('de-DE')}
						</p>
						{#if selectedEvent.displayedStart}
							<p>
								<strong>Anzeige Start:</strong>
								{new Date(selectedEvent.displayedStart).toLocaleString('de-DE')}
							</p>
						{/if}

						<h4>Zugehöriges XML-Snippet:</h4>
						<pre class="xml-box"><code>{selectedEvent.xmlString}</code></pre>
					</div>
				{/if}
			</div>
		{/if}
	</main>
</div>

<style>
	:global(html, body) {
		margin: 0;
		padding: 0;
		height: 100vh;
		overflow: hidden;
		font-family: sans-serif;
	}

	.app-container {
		display: grid;
		grid-template-columns: 1fr;
		height: 100vh;
		transition: grid-template-columns 0.4s ease;
	}

	.app-container.has-files {
		grid-template-columns: 260px 1fr;
	}

	.app-container.has-files.collapsed {
		grid-template-columns: 80px 1fr;
	}

	.sidebar-area {
		display: flex;
		flex-direction: column;
		background: #f8f9fa;
		border-right: 1px solid #ddd;
		padding: 20px;
		box-sizing: border-box;
		justify-content: center;
		align-items: center;
	}

	.app-container.has-files .sidebar-area {
		justify-content: flex-start;
		align-items: stretch;
		padding: 15px;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 20px;
	}

	.file-list {
		flex-grow: 1;
		overflow-y: auto;
		margin-bottom: 15px;
	}

	.file-item {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 10px;
		background: white;
		border: 1px solid #eee;
		border-radius: 6px;
		margin-bottom: 8px;
		cursor: pointer;
		text-align: left;
	}

	.file-item.active {
		border-color: #2196f3;
		background: #e8f4fd;
		font-weight: bold;
	}

	.dropzone {
		border: 2px dashed #bbb;
		border-radius: 8px;
		background: white;
		display: flex;
		align-items: center;
		justify-content: center;
		text-align: center;
		transition: all 0.3s ease;
	}

	.app-container:not(.has-files) .dropzone {
		width: 400px;
		height: 250px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.05);
	}

	.app-container.has-files .dropzone {
		width: 100%;
		height: auto;
		padding: 15px;
		margin-top: auto;
	}

	.app-container.has-files.collapsed .dropzone {
		border: none;
		background: transparent;
		padding: 0;
	}

	.btn {
		display: inline-block;
		background: #2196f3;
		color: white;
		padding: 8px 16px;
		border-radius: 4px;
		margin-top: 10px;
		cursor: pointer;
	}

	.btn-mini {
		background: #e2e8f0;
		padding: 8px;
		border-radius: 4px;
		display: block;
		text-align: center;
		font-size: 12px;
		cursor: pointer;
	}

	.main-content {
		background: #f1f3f5;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.viewer {
		background: white;
		padding: 40px;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
	}

	.viewer {
		width: 100%;
		max-width: 1200px;
		height: 90vh;
		overflow-y: auto;
		padding: 20px;
		background: white;
		border-radius: 8px;
	}

	.event-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 15px;
		margin-top: 20px;
		margin-bottom: 30px;
	}

	.event-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		padding: 12px;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		min-height: 110px;
	}

	.event-card:hover {
		border-color: #cbd5e1;
		background: #f1f5f9;
	}

	.event-card.active {
		border-color: #3b82f6;
		background: #eff6ff;
	}

	.event-card.has-error {
		border-color: #f87171;
		background: #fef2f2;
	}

	.card-top {
		display: flex;
		justify-content: space-between;
		font-size: 10px;
		color: #64748b;
	}

	.error-badge {
		background: #fee2e2;
		color: #dc2626;
		padding: 1px 5px;
		border-radius: 4px;
		font-weight: bold;
		text-transform: uppercase;
	}

	.event-title {
		font-size: 15px;
		margin: 8px 0;
		color: #1e293b;
		font-weight: 600;
	}

	.card-bottom {
		font-size: 12px;
		color: #475569;
	}

	.detail-panel {
		border-top: 2px solid #3b82f6;
		background: #f8fafc;
		padding: 20px;
		border-radius: 8px;
	}

	.xml-box {
		background: #1e293b;
		color: #38bdf8;
		padding: 15px;
		border-radius: 6px;
		overflow-x: auto;
		font-family: monospace;
		font-size: 11px;
		max-height: 250px;
	}

	.error-banner {
		background: #fee2e2;
		border: 1px solid #f87171;
		color: #b91c1c;
		padding: 10px;
		border-radius: 6px;
		margin-bottom: 15px;
	}
</style>
