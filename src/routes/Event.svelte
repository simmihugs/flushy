<script lang="ts">
	import { formatTime, formatMilliseconds, sendeTag } from './helper';
	let { event, active = false, onSelect } = $props();

	function explainError(e: any) {
		if (e.gap) {
			return 'Gap';
		} else {
			return 'Overlap';
		}
	}

	function showError(e: any) {
		if (e.gap) {
			return formatMilliseconds(e.gap);
		} else {
			return formatMilliseconds(e.overlap);
		}
	}
</script>

<span>
	<div
		class="event-card"
		class:active
		class:has-error={event.hasError}
		onclick={onSelect}
		role="button"
		tabindex="0"
	>
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
				Sendetag {sendeTag(event.startTime)}
			</span>
			<span class="time">
				{formatTime(event.startTime)} - {formatTime(event.endTime)}
			</span>
		</div>
		<div class="error-indicators">
			{#if event.errorBack}
				<span class="error-badge back">⚠️ Back Error: {explainError(event.errorBack)}</span>
			{/if}
		</div>
	</div>
	<div class="indent">
		{#if event.errorBack}
			<span class="error-badge back">{showError(event.errorBack)}</span>
		{/if}
	</div>
</span>

<style>
	.indent {
		padding-left: 25px;
	}
	.event-card {
		background: var(--event-card-background);
		border: 1px solid var(--event-card-border);
		border-radius: 8px;
		padding: 12px;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s ease;
		display: flex;
		flex-direction: column;
		justify-content: space-between;

		min-height: 110px;

		width: 100%;
		margin: 8px 0;
		box-sizing: border-box;
	}

	.event-card:hover {
		background: var(--event-card-background-hover);
		border: 1px solid var(--event-card-border-hover);
	}

	.event-card.active {
		background: var(--event-card-background-active);
		border-color: var(--event-card-border-active);
	}

	.event-card.has-error {
		background: var(--event-card-background-has-error);
		border-color: var(--event-card-border-has-error);
	}

	.event-card.has-error:hover {
		background: var(--event-card-background-has-error-hover);
		border-color: var(--event-card-border-has-error-hover);
	}

	.event-card.has-error.active {
		background: var(--event-card-background-has-error);
		border: 2px solid var(--event-card-border-active);
		box-shadow: 0 0 8px rgba(59, 130, 246, 0.5);
	}

	.card-top {
		display: flex;
		justify-content: space-between;
		font-size: 10px;
		color: var(--event-card-top);
	}

	.error-badge {
		background: var(--event-card-error-badge-background);
		color: var(--event-card-error-badge-color);
		padding: 1px 5px;
		border-radius: 4px;
		font-weight: bold;
	}

	.event-title {
		font-size: 15px;
		margin: 8px 0;
		color: var(--event-card-title);
		font-weight: 600;
	}

	.card-bottom {
		font-size: 12px;
		color: var(--event-card-bottom);
		display: flex;
		flex-direction: column;
	}

	.error-indicators {
		display: flex;
		gap: 4px;
	}
</style>
