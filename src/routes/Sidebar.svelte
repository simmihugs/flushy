<script lang="ts">
	//import type { MyFile } from './MyFile.ts';
	let { files, activeFile = $bindable(), onFileSelect } = $props();
</script>

<aside class="sidebar">
	<div class="file-list">
		{#if files}
			{#each files as file}
				<button class:active={activeFile?.path === file.path} onclick={() => onFileSelect(file)}>
					{file.name}
				</button>
			{/each}
		{/if}
	</div>

	<div class="drop-container">
		<div class="dropzone">
			<p>Dateien (.xml, .pts) hierher ziehen</p>
			<span class="or-separator">oder</span>
			<button type="button" onclick={onFileSelect}>Datei auswählen</button>
		</div>
	</div>
</aside>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		height: 100%;
		padding: 20px;
	}

	.file-list {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

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
		cursor: pointer;
	}

	button:hover {
		border-color: #396cd8;
	}

	button:active {
		border-color: #396cd8;
		background-color: #e8e8e8;
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

	@media (prefers-color-scheme: dark) {
		button {
			color: #ffffff;
			background-color: #0f0f0f98;
		}
		button:active {
			background-color: #0f0f0f69;
		}
		.dropzone {
			background-color: #1e1e1e;
			border-color: #24c8db;
		}
		.or-separator {
			color: #aaa;
		}
	}
</style>
