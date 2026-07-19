<script lang="ts">
	import Event from './Event.svelte';
	import CollapsedCard from './CollapsedCard.svelte';

	let { showCollapsed = $bindable(), activeFile, selectedEvent = $bindable() } = $props();
	let groupedEvents = $derived.by(() => {
		if (activeFile == null || !activeFile.events) {
			return [];
		}

		if (!showCollapsed) {
			return activeFile.events.map((event) => ({
				type: event.hasError ? 'error' : 'ok-flat',
				data: event
			}));
		}

		const result = [];
		let currentOkGroup = null;

		for (const event of activeFile.events) {
			if (event.hasError) {
				if (currentOkGroup) {
					result.push(currentOkGroup);
					currentOkGroup = null;
				}
				result.push({ type: 'error', data: event });
			} else {
				if (!currentOkGroup) {
					currentOkGroup = {
						type: 'collapsed-ok',
						count: 1,
						startTime: event.startTime,
						endTime: event.endTime,
						events: [event]
					};
				} else {
					currentOkGroup.count++;
					currentOkGroup.endTime = event.endTime;
					currentOkGroup.events.push(event);
				}
			}
		}

		if (currentOkGroup) {
			result.push(currentOkGroup);
		}

		return result;
	});
</script>

<div>
	{#if groupedEvents.length > 0}
		<div class="filter-area">
			<label class="switch-container">
				<input type="checkbox" bind:checked={showCollapsed} />
				<span class="switch-slider"></span>
				<span class="switch-label">Fehlerfreie Blöcke zusammenfassen</span>
			</label>
		</div>
	{/if}

	<div class="event-list">
		{#each groupedEvents as item}
			{#if item.type === 'error'}
				<Event
					event={item.data}
					active={selectedEvent?.eventId === item.data.eventId}
					onSelect={() => (selectedEvent = item.data)}
				/>
			{:else if item.type === 'ok-flat'}
				<Event
					event={item.data}
					active={selectedEvent?.eventId === item.data.eventId}
					onSelect={() => (selectedEvent = item.data)}
				/>
			{:else}
				<CollapsedCard {item} />
			{/if}
		{:else}
			<div class="empty-state">
				<p>Es wurden noch keine XML-Events geladen.</p>
			</div>
		{/each}
	</div>
</div>

<style>
	.filter-area {
		display: flex;
		justify-content: center;
		padding: 10px 0;
		margin-bottom: 15px;
	}

	.switch-container {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		gap: 10px;
		user-select: none;
		font-size: 14px;
		color: #475569;
	}

	.switch-container input {
		display: none;
	}

	.switch-slider {
		position: relative;
		width: 40px;
		height: 20px;
		background-color: #cbd5e1;
		border-radius: 20px;
		transition: background-color 0.2s;
	}

	.switch-slider::before {
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

	.switch-container input:checked + .switch-slider {
		background-color: #3b82f6;
	}

	.switch-container input:checked + .switch-slider::before {
		transform: translateX(20px);
	}

	.empty-state {
		text-align: center;
		padding: 20px;
		color: #666;
	}

	@media (prefers-color-scheme: dark) {
		.switch-container {
			color: #94a3b8;
		}
		.switch-slider {
			background-color: #475569;
		}
		.switch-container input:checked + .switch-slider {
			background-color: #24c8db;
		}
	}
</style>
