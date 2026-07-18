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

<svelte:window on:dragover|preventDefault on:drop|preventDefault />

<div
  class="app-container"
  class:has-files={files.length > 0}
  class:collapsed={isSidebarCollapsed}
>
  <aside class="sidebar-area">
    {#if files.length > 0}
      <div class="sidebar-header" transition:fade>
        <button on:click={() => (isSidebarCollapsed = !isSidebarCollapsed)}>
          {isSidebarCollapsed ? "➔" : "✕"}
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
            {isSidebarCollapsed ? "＋" : "＋ Datei hinzufügen"}
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
        <h2>Inhalt von: {activeFile?.name}</h2>
        <p>Größe: {((activeFile?.size || 0) / 1024).toFixed(2)} KB</p>
      </div>
    {/if}
  </main>
</div>

<style>
  /* CSS bleibt weitgehend identisch, ist nun aber sauber auf die Komponente gekapselt! */
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
</style>
