<script lang="ts">
	import { formatTime } from './helper';
	let { item } = $props();
</script>

<div class="collapsed-card">
	<span class="ok-badge">✓ OK</span>
	<span class="collapsed-text">
		{item.count} fehlerfreie Sendungen übersprungen
	</span>
	<span class="collapsed-time">
		{formatTime(item.startTime)} - {formatTime(item.endTime)}
	</span>
</div>

<style>
	.collapsed-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		background: #ffffff;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		padding: 10px 20px;
		margin: 8px auto;
		width: 80%;
		max-width: 600px;
		font-size: 13px;
		color: #475569;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
	}

	.ok-badge {
		background: #dcfce7;
		color: #15803d;
		padding: 2px 6px;
		border-radius: 4px;
		font-weight: bold;
		font-size: 11px;
	}

	.collapsed-text {
		font-weight: 500;
	}

	.collapsed-time {
		color: #64748b;
	}
</style>
