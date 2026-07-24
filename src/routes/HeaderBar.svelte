<script lang="ts">
	let { files, toggleTheme, getTheme, showCollapsed = $bindable() } = $props();
</script>

<header class="header-bar">
	<div class="header-content">
		{#if files.length > 0}
			<div class="filter-area">
				<label class="filter-container">
					<input type="checkbox" bind:checked={showCollapsed} />
					<span class="filter-slider"></span>
					<span class="filter-label">Fehlerfreie Blöcke zusammenfassen</span>
				</label>
			</div>
		{/if}
		<button class="theme-switch-button" onclick={toggleTheme}>{getTheme()}</button>
	</div>
</header>

<style>
	.header-bar {
		width: 100%;
		height: 50px;
		display: flex;
		justify-content: flex-end;
		align-items: center;
		padding: 0 20px;
		box-sizing: border-box;
		flex-shrink: 0;
	}

	.header-content {
		display: flex;
		align-items: center;
		gap: 20px;
	}

	.theme-switch-button {
		padding: 6px 12px;
		border-radius: 8px;
		cursor: pointer;

		background: var(--theme-switch-background);
		color: var(--theme-switch-color);
		border: 1px solid var(--theme-switch-border);
		transition:
			border-color 0.25s ease,
			background-color 0.25s ease;

		box-shadow: 0 1px 1px rgba(0, 0, 0, 0.1);
	}

	.theme-switch-button:hover {
		border-color: var(--theme-switch-border-hover);
	}

	.filter-area {
		display: flex;
		align-items: center;
	}

	.filter-container {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		gap: 10px;
		user-select: none;
		font-size: 14px;
		color: var(--theme-filter-color, #475569);
	}

	.filter-container input {
		display: none;
	}

	.filter-slider {
		position: relative;
		width: 40px;
		height: 20px;
		background-color: #cbd5e1;
		border-radius: 20px;
		transition: background-color 0.2s;
	}

	.filter-slider::before {
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

	.filter-container input:checked + .filter-slider {
		background-color: #3b82f6;
	}

	.filter-container input:checked + .filter-slider::before {
		transform: translateX(20px);
	}
</style>
