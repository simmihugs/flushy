<script lang="ts">
	import { ArrowBigLeft, Settings } from '@lucide/svelte';
	let { showConfigPage = $bindable() } = $props();
</script>

<div class="statusbar">
	{#if showConfigPage}
		<div title="leave config...">
			<ArrowBigLeft size={20} onclick={() => (showConfigPage = false)} />
		</div>
	{:else}
		<div title="goto config">
			<Settings size={20} onclick={() => (showConfigPage = true)} />
		</div>
	{/if}
</div>

<style>
	.statusbar {
		border-top: 1px solid var(--mode-inactive-color);
		padding-left: 5px;
		padding-top: 5px;
	}
</style>
