<script lang="ts">
	import { setConfigEntry } from '$lib/config';
	import type { ConfigValue, ConfigNum, ConfigEntryId, ConfigRange } from '$lib/models';
	import { onMount } from 'svelte';
	import ResetConfigButton from './ResetConfigButton.svelte';

	export let entryId: ConfigEntryId;

	let content = entryId.entry.value.content as ConfigNum;
	let range = content.range as ConfigRange;
	let type = entryId.entry.value.type as 'int32' | 'double' | 'single';

	$: fillPercent = clamp(((content.value - range.start) / (range.end - range.start)) * 100, 0, 100);

	let element: HTMLDivElement;
	let fill: HTMLDivElement;
	let handle: HTMLDivElement;

	let isDragging = false;
	let inputString = content.value.toString();

	function onReset(newValue: ConfigValue) {
		content = newValue.content as ConfigNum;
		inputString = content.value.toString();
	}

	function submitValue() {
		setConfigEntry(entryId, { type, content });
	}

	const DECIMAL_STEP = 0.1;
	const X_OFFSET = 16;

	function calculateNewValue(clientX: number) {
		let rect = element.getBoundingClientRect();
		rect.width -= X_OFFSET;
		rect.x += X_OFFSET / 2;

		let x = clientX - rect.left;
		let newValue = range.start + (range.end - range.start) * (x / rect.width);

		if (type === 'double' || type == 'single') {
			newValue = Math.round(newValue / DECIMAL_STEP) * DECIMAL_STEP;
		} else if (type === 'int32') {
			newValue = Math.round(newValue);
		}

		newValue = clamp(newValue, range.start, range.end);
		inputString = newValue.toString();
		content.value = newValue;
	}

	function clamp(value: number, min: number, max: number) {
		return Math.max(min, Math.min(max, value));
	}
</script>

<svelte:window
	on:mousemove={(e) => {
		if (isDragging) {
			calculateNewValue(e.clientX);
		}
	}}
	on:mouseup={(e) => {
		if (isDragging) {
			isDragging = false;
			calculateNewValue(e.clientX);
			submitValue();
		}
	}}
/>

<div
	class="group h-5 flex-grow rounded-full bg-gray-900 py-1 pl-1 pr-3"
	role="slider"
	aria-valuemin={range.start}
	aria-valuemax={range.end}
	aria-valuenow={content.value}
	tabindex="0"
	bind:this={element}
	on:keydown={(e) => {
		if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
			content.value = Math.max(range.start, content.value - 1);
		} else if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
			content.value = Math.min(range.end, content.value + 1);
		}

		inputString = content.value.toString();
	}}
	on:mousedown={(e) => {
		isDragging = true;
		calculateNewValue(e.clientX);
	}}
>
	<div
		class="relative h-full min-w-1 rounded-l-full group-hover:bg-gray-600"
		style="width: {fillPercent}%;"
		class:bg-gray-700={!isDragging}
		class:bg-gray-600={isDragging}
		bind:this={fill}
	>
		<div
			class="absolute right-[-0.5rem] h-3 w-3 rounded-full"
			class:bg-gray-400={!isDragging}
			class:bg-gray-300={isDragging}
			bind:this={handle}
			draggable="false"
		/>
	</div>
</div>

<input
	type="number"
	bind:value={inputString}
	on:change={() => {
		let newValue = parseFloat(inputString);

		if (!isNaN(newValue)) {
			newValue = clamp(newValue, range.start, range.end);

			if (type === 'int32') {
				newValue = Math.round(newValue);
			}

			content.value = newValue;
			submitValue();
		}

		inputString = content.value.toString();
	}}
	class="ml-3 w-1/6 min-w-0 flex-shrink rounded-lg border border-slate-500 border-opacity-0 bg-gray-900
		 px-3 py-1
		 text-slate-300 placeholder-slate-400 hover:border-opacity-100
		   hover:text-slate-200 focus:border-opacity-0 focus:outline-none focus:ring-2 focus:ring-green-400"
/>
<ResetConfigButton {entryId} {onReset} />

<style>
	input[type='number']::-webkit-inner-spin-button,
	input[type='number']::-webkit-outer-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}
</style>
