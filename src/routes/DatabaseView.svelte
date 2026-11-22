<script lang="ts">
	import type { Param } from './Param.ts';
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { Database, RefreshCw, Search } from '@lucide/svelte';

	$effect(() => {
		if (configParams) {
			void loadEntries();
		}
	});

	interface StoredEntry {
		content_id: string;
		runtime_ms: number | null;
		first_seen: string;
	}

	interface DbListResponse {
		total: number;
		items: StoredEntry[];
	}

	let { configParams = null as Param[] | null } = $props();

	let items = $state<StoredEntry[]>([]);
	let total = $state(0);
	let search = $state('');
	let loading = $state(false);
	let errorMsg = $state<string | null>(null);

	function getDbName(): string {
		const val = configParams?.find((p) => p.name === 'DATABASE_URL')?.value?.trim();
		return val && val.length > 0 ? val : 'ses2.db';
	}

	function formatRuntime(ms: number | null): string {
		if (ms === null || Number.isNaN(ms)) return '–';
		const totalSeconds = Math.floor(ms / 1000);
		const millis = ms % 1000;
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const seconds = totalSeconds % 60;
		const hh = String(hours).padStart(2, '0');
		const mm = String(minutes).padStart(2, '0');
		const ss = String(seconds).padStart(2, '0');
		return millis > 0
			? `${hh}:${mm}:${ss}.${String(millis).padStart(3, '0')}`
			: `${hh}:${mm}:${ss}.000`;
	}

	async function loadEntries() {
		loading = true;
		errorMsg = null;
		try {
			const res = await invoke<DbListResponse>('list_db_entries', {
				db: getDbName(),
				search: search.trim() || null,
				limit: 300,
				offset: 0
			});
			total = res.total;
			items = res.items;
		} catch (err) {
			errorMsg = String(err);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void loadEntries();
	});
</script>

<div class="db-view">
	<div class="db-header">
		<div class="title-group">
			<Database size={20} />
			<h1>Database</h1>
			<span class="badge">{items.length} / {total}</span>
		</div>

		<div class="controls">
			<div class="search-box">
				<Search size={15} />
				<input
					type="text"
					placeholder="ContentId filtern…"
					bind:value={search}
					onkeydown={(e) => {
						if (e.key === 'Enter') void loadEntries();
					}}
				/>
			</div>
			<button type="button" class="btn" onclick={() => void loadEntries()} disabled={loading}>
				<RefreshCw size={15} />
				<span>Neu laden</span>
			</button>
		</div>
	</div>

	{#if errorMsg}
		<div class="error-banner">{errorMsg}</div>
	{:else if loading && items.length === 0}
		<div class="muted">Lade Einträge…</div>
	{:else if items.length === 0}
		<div class="muted">Keine Einträge vorhanden.</div>
	{:else}
		<div class="table-wrap">
			<table>
				<thead>
					<tr>
						<th>ContentId</th>
						<th>Runtime</th>
						<th>First Seen</th>
					</tr>
				</thead>
				<tbody>
					{#each items as entry (entry.content_id)}
						<tr>
							<td class="mono">{entry.content_id}</td>
							<td class="mono">{formatRuntime(entry.runtime_ms)}</td>
							<td class="mono muted">{entry.first_seen}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.db-view {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		padding: 1rem;
		width: 100%;
		height: 95%;
		overflow: hidden;
	}

	.db-header {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
	}

	.title-group {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.title-group h1 {
		margin: 0;
		font-size: 1.25rem;
	}

	.badge {
		font-family: monospace;
		font-size: 0.8rem;
		padding: 0.15rem 0.5rem;
		border-radius: 999px;
		border: 1px solid var(--event-card-border, #334155);
		background: var(--event-card-background, transparent);
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.search-box {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.25rem 0.5rem;
		border-radius: 6px;
		border: 1px solid var(--event-card-border-hover, #475569);
		background: var(--input-bg, #1e1e1e);
		color: var(--input-text, inherit);
	}

	.search-box input {
		border: none;
		outline: none;
		background: transparent;
		color: inherit;
		font-family: monospace;
		font-size: 0.85rem;
		width: 14rem;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.35rem 0.65rem;
		border-radius: 6px;
		border: 1px solid var(--event-card-border-hover, #475569);
		background: var(--event-card-background, transparent);
		color: inherit;
		font-size: 0.85rem;
		cursor: pointer;
	}

	.btn:hover:not(:disabled) {
		background: var(--event-card-background-hover, rgba(255, 255, 255, 0.08));
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.table-wrap {
		flex: 1;
		min-height: 0;
		overflow: auto;
		border: 1px solid var(--event-card-border, #334155);
		border-radius: 6px;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.85rem;
	}

	th,
	td {
		text-align: left;
		padding: 0.45rem 0.7rem;
		border-bottom: 1px solid var(--event-card-border, #334155);
	}

	th {
		position: sticky;
		top: 0;
		background: var(--bg-color, #2f2f2f);
		font-weight: 600;
		z-index: 1;
	}

	tr:hover td {
		background: var(--event-card-background-hover, rgba(255, 255, 255, 0.05));
	}

	.mono {
		font-family: monospace;
	}

	.muted {
		opacity: 0.7;
	}

	.error-banner {
		padding: 0.6rem 0.8rem;
		border-radius: 6px;
		background: var(--event-card-error-badge-background, #7f1d1d);
		color: var(--event-card-error-badge-color, #fca5a5);
		font-size: 0.85rem;
	}
</style>
