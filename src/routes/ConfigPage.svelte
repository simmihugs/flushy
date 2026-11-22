<script lang="ts">
	import type { Param } from './Param.ts';
	import { invoke } from '@tauri-apps/api/core';
	import { Save, PenIcon, SquareStopIcon, Eye, EyeOff } from '@lucide/svelte';

	let { configParams = $bindable<Param[] | null>(null), configPath = null as string | null } =
		$props();

	let editingIndex = $state<number | null>(null);
	let draftValue = $state('');
	let hasChanged = $state(false);
	let errorMsg = $state<string | null>(null);
	let saving = $state(false);
	let revealed = $state<Record<string, boolean>>({});

	function startEdit(index: number, param: Param) {
		errorMsg = null;
		editingIndex = index;
		draftValue = param.value;
	}

	function handleAbort() {
		editingIndex = null;
		draftValue = '';
		errorMsg = null;
	}

	async function handleUpdate(index: number) {
		if (!configParams || editingIndex === null) return;

		const updated = configParams.map((item, i) =>
			i === index ? { ...item, value: draftValue } : item
		);

		saving = true;
		errorMsg = null;
		try {
			await invoke('save_config', {
				path: configPath,
				params: updated
			});
			configParams = updated;
			hasChanged = true;
			editingIndex = null;
			draftValue = '';
		} catch (err) {
			errorMsg = String(err);
		} finally {
			saving = false;
		}
	}

	function toggleReveal(name: string) {
		revealed[name] = !revealed[name];
	}
</script>

<div class="fileview">
	<h1>Configuration Page</h1>
	<div class="event-list">
		<div class="param header">
			<span>Name</span>
			<span>Value</span>
			<span>Aktionen</span>
		</div>

		{#if errorMsg}
			<div class="error-banner">{errorMsg}</div>
		{/if}

		{#if !configParams}
			<div class="empty">Konfiguration wird geladen…</div>
		{:else}
			{#each configParams as param, index (param.name)}
				<div class="param">
					<span class="param-name">{param.name}</span>

					{#if editingIndex === index}
						<input
							type="text"
							bind:value={draftValue}
							disabled={saving}
							onkeydown={(e) => {
								if (e.key === 'Enter') void handleUpdate(index);
								if (e.key === 'Escape') handleAbort();
							}}
						/>
						<div class="actions">
							<button
								type="button"
								class="icon-btn"
								onclick={handleAbort}
								disabled={saving}
								title="Abbrechen"
							>
								<SquareStopIcon size={16} />
							</button>
							<button
								type="button"
								class="icon-btn"
								onclick={() => handleUpdate(index)}
								disabled={saving}
								title="Speichern"
							>
								<Save size={16} />
							</button>
						</div>
					{:else}
						<span class="param-value">
							{revealed[param.name] ? param.value : '*****'}
						</span>
						<div class="actions">
							<button
								type="button"
								class="icon-btn"
								onclick={() => toggleReveal(param.name)}
								title={revealed[param.name] ? 'Verbergen' : 'Anzeigen'}
							>
								{#if revealed[param.name]}
									<EyeOff size={16} />
								{:else}
									<Eye size={16} />
								{/if}
							</button>
							<button
								type="button"
								class="icon-btn"
								onclick={() => startEdit(index, param)}
								disabled={editingIndex !== null || saving}
								title="Bearbeiten"
							>
								<PenIcon size={16} />
							</button>
						</div>
					{/if}
				</div>
			{/each}
		{/if}
	</div>
</div>

<style>
	.fileview {
		padding: 1rem;
		width: 100%;
		overflow-y: auto;
	}

	.event-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-width: 100%;
	}

	.param {
		display: grid;
		grid-template-columns: 12rem 1fr auto;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--event-card-border, #e2e8f0);
		border-radius: 6px;
		background: var(--event-card-background, transparent);
	}

	.param.header {
		font-weight: 600;
		background: transparent;
		border: none;
		padding-bottom: 0.25rem;
	}

	.param-name {
		font-family: monospace;
		font-size: 0.9rem;
		word-break: break-all;
	}

	.param-value {
		font-family: monospace;
		font-size: 0.9rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	input[type='text'] {
		width: 100%;
		padding: 0.35rem 0.5rem;
		border-radius: 4px;
		border: 1px solid var(--event-card-border-hover, #cbd5e1);
		background: var(--input-bg, #ffffff);
		color: var(--input-text, inherit);
		font-family: monospace;
		font-size: 0.9rem;
	}

	.actions {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}

	.icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.3rem;
		border: 1px solid transparent;
		border-radius: 4px;
		background: transparent;
		color: inherit;
		cursor: pointer;
	}

	.icon-btn:hover:not(:disabled) {
		border-color: var(--event-card-border-hover, #cbd5e1);
	}

	.icon-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.error-banner {
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		background: var(--event-card-error-badge-background, #fee2e2);
		color: var(--event-card-error-badge-color, #dc2626);
		font-size: 0.875rem;
	}

	.empty {
		opacity: 0.7;
		font-size: 0.9rem;
	}
</style>
