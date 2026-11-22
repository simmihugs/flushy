<script lang="ts">
	import { ArrowBigLeft, Settings, Database, RefreshCw } from '@lucide/svelte';

	type DbStatus = 'unknown' | 'connected' | 'error';
	type SyncStatus = 'idle' | 'syncing' | 'synced' | 'error';

	let {
		showConfigPage = $bindable(false),
		showDbPage = $bindable(false),
		dbStatus = 'unknown' as DbStatus,
		syncStatus = 'idle' as SyncStatus,
		syncMessage = '',
		onResync = undefined as (() => void) | undefined
	} = $props();
</script>

<div class="statusbar">
	<div class="left">
		{#if showConfigPage}
			<button
				type="button"
				class="icon-btn"
				title="leave config..."
				onclick={() => (showConfigPage = false)}
			>
				<ArrowBigLeft size={20} />
			</button>
		{:else}
			<button
				type="button"
				class="icon-btn"
				title="goto config"
				onclick={() => (showConfigPage = true)}
			>
				<Settings size={20} />
			</button>
		{/if}
	</div>

	<div class="right">
		<div
			class="indicator status-{dbStatus}"
			title={dbStatus === 'error' ? syncMessage : `DB: ${dbStatus}`}
			onclick={() => {
				if (showDbPage) {
					showDbPage = false;
				} else {
					showDbPage = true;
				}
			}}
		>
			<Database size={16} />
			<span>DB</span>
		</div>

		<button
			type="button"
			class="indicator sync-btn status-{syncStatus}"
			title={syncMessage || `Sync: ${syncStatus}`}
			disabled={!onResync || syncStatus === 'syncing'}
			onclick={() => onResync?.()}
		>
			<span class="spin-wrap" class:spinning={syncStatus === 'syncing'}>
				<RefreshCw size={16} />
			</span>
			{#if syncMessage}
				<span class="msg">{syncMessage}</span>
			{/if}
		</button>
	</div>
</div>

<style>
	.statusbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		border-top: 1px solid var(--mode-inactive-color);
		padding: 5px 10px 5px 5px;
		min-height: 28px;
		font-size: 0.8rem;
	}

	.left,
	.right {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 2px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: inherit;
		cursor: pointer;
	}

	.icon-btn:hover {
		background: var(--event-card-background-hover, rgba(255, 255, 255, 0.08));
	}

	.indicator {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-family: monospace;
		font-size: 0.78rem;
	}

	.sync-btn {
		border: 1px solid transparent;
		border-radius: 4px;
		padding: 2px 6px;
		background: transparent;
		color: inherit;
		cursor: pointer;
	}

	.sync-btn:hover:not(:disabled) {
		border-color: var(--event-card-border-hover, #475569);
	}

	.sync-btn:disabled {
		cursor: default;
		opacity: 0.85;
	}

	.msg {
		max-width: 24rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.status-connected,
	.status-synced {
		color: #22c55e;
	}

	.status-error {
		color: #ef4444;
	}

	.status-syncing {
		color: #38bdf8;
	}

	.status-unknown,
	.status-idle {
		color: var(--mode-inactive-color);
	}

	.spin-wrap {
		display: inline-flex;
		align-items: center;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
