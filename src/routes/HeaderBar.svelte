<script lang="ts">
	let {
		files,
		toggleTheme,
		getTheme,
		activeFile,
		showVaEvents = $bindable(),
		showCollapsed = $bindable()
	} = $props();

	let show = $state(false);
</script>

<header class="header-bar">
	<div class="header-content">
		{#if files.length > 0}
			<div class="filter-area">
				<div
					class="tooltip-wrapper"
					onmouseenter={() => (show = true)}
					onmouseleave={() => (show = false)}
				>
					<div
						class="icon-button"
						onclick={() => {
							showCollapsed = showCollapsed ? false : true;
							console.log('showCollapsed: ', showCollapsed);
						}}
					>
						{#if showCollapsed}
							▶️
						{:else}
							🔽
						{/if}
					</div>

					{#if show}
						{#if showCollapsed}
							<div class="popup">Alle Blöcke anzeigen</div>
						{:else}
							<div class="popup">Fehlerfreie Blöcke zusammenfassen</div>
						{/if}
					{/if}
				</div>
				<button
					class="mode-button"
					class:inactive={showVaEvents}
					onclick={() => (showVaEvents = false)}
				>
					<span>
						Epg Events:
						<span class={activeFile.si_events_errors > 0 ? 'red' : 'green'}
							>{activeFile.si_events_errors}</span
						>
						Errors
					</span>
				</button>
				<button
					class="mode-button"
					class:inactive={!showVaEvents}
					onclick={() => (showVaEvents = true)}
				>
					<span>
						VaEvents:
						<span class={activeFile.va_events_errors > 0 ? 'red' : 'green'}
							>{activeFile.va_events_errors}</span
						>
						Errors
					</span>
				</button>
			</div>
		{/if}
		<button class="theme-switch-button" onclick={toggleTheme}>{getTheme()}</button>
	</div>
</header>

<style>
	.icon-button {
		background: transparent;
		cursor: pointer;
		margin-left: 4px;
		margin-right: 4px;
	}

	.red {
		color: tomato;
	}

	.green {
		color: green;
	}

	.mode-button {
		margin-left: 4px;
		margin-right: 4px;
		padding: 6px 12px;
		border-radius: 8px;
		cursor: pointer;

		background: var(--mode-background);
		color: var(--mode-color);
		border: 1px solid var(--mode-border);
		box-shadow: 0 1px 1px rgba(0, 0, 0, 0.1);
	}

	.mode-button:hover {
		border-color: var(--mode-border-hover);
	}

	.mode-button.inactive {
		background: var(--mode-inactive-background);
		color: var(--mode-inactive-color);
		border: 1px solid var(--mode-inactive-border);
	}

	.mode-button.inactive:hover {
		border-color: var(--mode-inactive-border-hover);
	}

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

	.tooltip-container {
		position: relative;
		display: inline-block;
	}

	.tooltip-wrapper {
		position: relative;
		display: inline-block;
	}

	.popup {
		position: absolute;
		top: calc(100% + 8px);
		left: 50%;
		background: var(--mode-background);
		color: var(--mode-color);
		padding: 6px 10px;
		border-radius: 4px;
		font-size: 0.8rem;
		white-space: nowrap;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
		z-index: 10;
	}
</style>
