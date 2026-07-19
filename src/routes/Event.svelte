<script lang="ts">
	import { formatTime } from './helper';
	let { event, active = false, onSelect } = $props();
</script>

<div class="event-card" class:active class:has-error={event.hasError} onclick={onSelect}>
	<div class="error-indicators">
		{#if event.errorFront}
			<span class="error-badge front">⚠️ Front Error: {event.errorFront}</span>
		{/if}
	</div>

	<div class="card-top">
		<span class="event-id">{event.eventId}</span>
	</div>

	<h3 class="event-title">{event.title}</h3>

	<div class="card-bottom">
		<span class="time">
			{formatTime(event.startTime)} - {formatTime(event.endTime)}
		</span>
	</div>
	<div class="error-indicators">
		{#if event.errorBack}
			<span class="error-badge back">⚠️ Back Error: {event.errorBack}</span>
		{/if}
	</div>
</div>

<style>
	.event-card {
		background: #f8fafc;
		border: 1px solid #e2e8f0;
		border-radius: 8px;
		padding: 12px;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		justify-content: space-between;

		min-height: 110px;
		/* max-width: 700px; */
		/* margin: 8px auto; */

		width: 100%;
		margin: 8px 0;
		box-sizing: border-box;
	}

	.event-card:hover {
		border-color: #cbd5e1;
		background: #f1f5f9;
	}

	.event-card.active {
		border-color: #3b82f6;
		background: #eff6ff;
	}

	.event-card.has-error {
		border-color: #f87171;
		background: #fef2f2;
	}

	.event-card.has-error:hover {
		border-color: #ed4747 !important;
		background: #f87171 !important;
	}

	.card-top {
		display: flex;
		justify-content: space-between;
		font-size: 10px;
		color: #64748b;
	}

	.error-badge {
		background: #fee2e2;
		color: #dc2626;
		padding: 1px 5px;
		border-radius: 4px;
		font-weight: bold;
	}

	.event-title {
		font-size: 15px;
		margin: 8px 0;
		color: #1e293b;
		font-weight: 600;
	}

	.card-bottom {
		font-size: 12px;
		color: #475569;
	}

	.error-indicators {
		display: flex;
		gap: 4px;
	}
</style>
