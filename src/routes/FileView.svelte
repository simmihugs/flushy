<script lang="ts">
	import Event from './Event.svelte';
	import CollapsedCard from './CollapsedCard.svelte';
	import DetailsComponent from './DetailsComponent.svelte';

	let { showVaEvents = $bindable(), showCollapsed = $bindable(), activeFile } = $props();

	let selectedEvent = $state(null);

	function toggleSelection(event) {
		if (selectedEvent?.eventId === event.eventId) {
			selectedEvent = null;
		} else {
			selectedEvent = event;
		}
		console.log(`selected event ${selectedEvent}`);
	}

	let groupedEvents = $derived.by(() => {
		if (activeFile == null || (!activeFile.si_events && !activeFile.va_events)) {
			return [];
		}

		let events = showVaEvents ? activeFile.va_events : activeFile.si_events;

		if (!showCollapsed) {
			return events.map((event) => ({
				type: event.hasError ? 'error' : 'ok-flat',
				data: event
			}));
		}

		const result = [];
		let currentOkGroup = null;

		for (const event of events) {
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

	function handleScroll(e) {
		if (e.target.scrollTop > 5) {
			selectedEvent = null;
		}
	}
</script>

<div class="fileview">
	<div class="event-list" onscroll={handleScroll}>
		{#each groupedEvents as item}
			{#if item.type === 'error'}
				<Event
					event={item.data}
					active={selectedEvent?.eventId === item.data.eventId}
					onSelect={() => toggleSelection(item.data)}
				/>
			{:else if item.type === 'ok-flat'}
				<Event
					event={item.data}
					active={selectedEvent?.eventId === item.data.eventId}
					onSelect={() => toggleSelection(item.data)}
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
	<div class="details">
		{#if selectedEvent}
			<DetailsComponent event={selectedEvent} />
		{/if}
	</div>
</div>

<style>
	.fileview {
		display: flex;
		flex: 1;
		width: 100%;
		height: 100%;
		min-width: 0;
		overflow: hidden;
	}
	.event-list {
		flex: 1;
		min-width: 400px;
		overflow-y: auto;
		padding: 8px 12px;
		box-sizing: border-box;
	}

	.details {
		max-width: 400px;
	}

	.empty-state {
		text-align: center;
		padding: 20px;
		color: #666;
	}
</style>
