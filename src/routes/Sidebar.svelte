<script lang="ts">
	let { files, activeFile = $bindable(), onFileSelect, isExpanded, onFileDelete } = $props();
</script>

<aside class="sidebar {isExpanded ? 'expanded' : 'centered'}">
	{#if isExpanded}
		<div class="file-list">
			{#each files as file}
				<div class="file-item" class:active={activeFile?.path === file.path}>
					<button class="file-select-btn" onclick={() => (activeFile = file)} title={file.name}>
						<span class="file-name">{file.name}</span>
					</button>

					<button
						class="delete-btn"
						onclick={(e) => {
							e.stopPropagation();
							onFileDelete(file.path);
						}}
						title="Datei entfernen"
					>
						🗑️
					</button>
				</div>
			{/each}
		</div>
	{/if}

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
		height: 100vh;
		padding: 20px;
		box-sizing: border-box;
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

	.drop-container {
		display: flex;
		justify-content: center;
		margin: 20px 0;
	}

	.sidebar.expanded .dropzone {
		width: 80%;
	}

	.or-separator {
		font-size: 0.9em;
		color: #666;
		font-style: italic;
	}

	.sidebar {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 300px;
		padding: 15px;
		box-sizing: border-box;
	}

	.file-list {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 20px;
	}

	.drop-container {
		flex-shrink: 0;
		padding-top: 10px;
		padding-bottom: 25px;
	}

	.dropzone {
		width: 100%;
		padding: 20px;
		border: 2px dashed #396cd8;
		border-radius: 12px;
		background-color: var(--input-bg);
		color: var(--text-color);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		box-sizing: border-box;
		transition:
			background-color 0.3s,
			color 0.3s;
	}

	.file-item {
		display: flex;
		align-items: center;
		background-color: var(--input-bg, #ffffff);
		border: 1px solid transparent;
		border-radius: 8px;
		overflow: hidden;
		box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
		transition: border-color 0.25s;
	}

	.file-item:hover {
		border-color: #396cd8;
	}

	.file-item.active {
		border-color: #396cd8;
	}

	.file-select-btn {
		flex: 1;
		background: none;
		border: none;
		box-shadow: none;
		text-align: left;
		padding: 0.6em 0.8em;
		border-radius: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.file-name {
		color: inherit;
	}

	.delete-btn {
		background: none;
		border: none;
		box-shadow: none;
		padding: 0.6em 0.8em;
		cursor: pointer;
		opacity: 0.6;
		transition:
			opacity 0.2s,
			transform 0.1s;
		border-radius: 0;
	}

	.delete-btn:hover {
		opacity: 1;
		transform: scale(1.1);
	}

	.file-item {
		display: flex;
		align-items: center;
		background-color: var(--input-bg, #ffffff);
		color: var(--text-color, #0f0f0f);
		border: 1px solid var(--event-card-border, #e2e8f0);
		border-radius: 8px;
		overflow: hidden;
		transition: all 0.2s ease;
	}

	.file-item:hover {
		border-color: #396cd8;
	}

	.file-item.active {
		border-color: #4073c9;
		color: #396cd8;
	}

	.file-select-btn {
		flex: 1;
		background: transparent;
		border: none;
		color: inherit;
		text-align: left;
		padding: 0.6em 0.8em;
		font-size: 0.9em;
		cursor: pointer;

		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.delete-btn {
		background: transparent;
		border: none;
		padding: 0.6em 0.8em;
		cursor: pointer;
		font-size: 1.1em;

		opacity: 0.7;
		filter: drop-shadow(0 0 1px rgba(0, 0, 0, 0.5));
		transition:
			opacity 0.2s,
			transform 0.1s;
	}

	.file-item.active .delete-btn {
		opacity: 0.9;
	}

	.delete-btn:hover {
		opacity: 1 !important;
		transform: scale(1.15);
	}
</style>
