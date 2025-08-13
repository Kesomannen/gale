<script lang="ts" generics="T, Key = number">
	import { run } from 'svelte/legacy';

	// (c) 2018 Rich Harris
	// https://github.com/sveltejs/svelte-virtual-list/blob/master/LICENSE
	import { onMount, tick, type Snippet } from 'svelte';

	type Props = {
		// props
		items: T[];
		height?: string;
		itemHeight?: number | undefined;
		// read-only, but visible to consumers via bind:start
		start?: number;
		end?: number;
		rowId?: (data: T) => Key;
		children?: Snippet<[{ item: T; index: number }]>;
	};

	let {
		items,
		height = '100%',
		itemHeight = undefined,
		start = $bindable(0),
		end = $bindable(0),
		rowId,
		children
	}: Props = $props();

	// local state
	let heightMap: number[] = [];
	let rows: HTMLCollectionOf<HTMLElement>;
	let viewport: HTMLElement = $state();
	let contents: HTMLElement = $state();
	let viewportHeight = $state(0);
	let visible: { index: number; data: T }[] = $derived(
		items.slice(start, end).map((data, i) => {
			return { index: i + start, data };
		})
	);
	let mounted: boolean = $state(false);

	let top = $state(0);
	let bottom = $state(0);
	let average_height: number;

	async function refresh(items: T[], viewportHeight: number, itemHeight: number | undefined) {
		const { scrollTop } = viewport;

		await tick(); // wait until the DOM is up to date

		let contentHeight = top - scrollTop;
		let i = start;

		while (contentHeight < viewportHeight && i < items.length) {
			let row = rows[i - start];

			if (!row) {
				end = i + 1;
				await tick(); // render the newly visible row
				row = rows[i - start];
			}

			const rowHeight = (heightMap[i] = itemHeight ?? row.offsetHeight);
			contentHeight += rowHeight;
			i += 1;
		}

		end = i;

		const remaining = items.length - end;
		average_height = (top + contentHeight) / end;

		bottom = remaining * average_height;
		heightMap.length = items.length;
	}

	async function handleScroll() {
		const { scrollTop } = viewport;

		const old_start = start;

		for (let v = 0; v < rows.length; v += 1) {
			heightMap[start + v] = itemHeight || rows[v].offsetHeight;
		}

		let i = 0;
		let y = 0;

		while (i < items.length) {
			const row_height = heightMap[i] || average_height;
			if (y + row_height > scrollTop) {
				start = i;
				top = y;

				break;
			}

			y += row_height;
			i += 1;
		}

		while (i < items.length) {
			y += heightMap[i] || average_height;
			i += 1;

			if (y > scrollTop + viewportHeight) break;
		}

		end = i;

		const remaining = items.length - end;
		average_height = y / end;

		while (i < items.length) heightMap[i++] = average_height;
		bottom = remaining * average_height;

		// prevent jumping if we scrolled up into unknown territory
		if (start < old_start) {
			await tick();

			let expected_height = 0;
			let actual_height = 0;

			for (let i = start; i < old_start; i += 1) {
				if (rows[i - start]) {
					expected_height += heightMap[i];
					actual_height += itemHeight || rows[i - start].offsetHeight;
				}
			}

			const d = actual_height - expected_height;
			if (d !== 0) {
				viewport.scrollTo(0, scrollTop + d);
			}
		}

		// TODO if we overestimated the space these
		// rows would occupy we may need to add some
		// more. maybe we can just call handle_scroll again?
	}

	export function scrollTo(y: number) {
		viewport.scrollTo(0, y);
	}

	// trigger initial refresh
	onMount(() => {
		rows = contents.getElementsByTagName(
			'svelte-virtual-list-row'
		) as HTMLCollectionOf<HTMLElement>;
		mounted = true;
	});

	// whenever `items` changes, invalidate the current heightmap
	run(() => {
		if (mounted) refresh(items, viewportHeight, itemHeight);
	});
</script>

<svelte-virtual-list-viewport
	bind:this={viewport}
	bind:offsetHeight={viewportHeight}
	onscroll={handleScroll}
	style="height: {height};"
>
	<svelte-virtual-list-contents
		bind:this={contents}
		style="padding-top: {top}px; padding-bottom: {bottom}px;"
	>
		{#each visible as row (rowId?.(row.data) ?? row.index)}
			<svelte-virtual-list-row>
				{#if children}{@render children({ item: row.data, index: row.index })}{:else}Missing
					template{/if}
			</svelte-virtual-list-row>
		{/each}
	</svelte-virtual-list-contents>
</svelte-virtual-list-viewport>

<style>
	svelte-virtual-list-viewport {
		position: relative;
		overflow-y: scroll;
		-webkit-overflow-scrolling: touch;
		display: block;
	}

	svelte-virtual-list-contents,
	svelte-virtual-list-row {
		display: block;
	}

	svelte-virtual-list-row {
		overflow: hidden;
	}
</style>
