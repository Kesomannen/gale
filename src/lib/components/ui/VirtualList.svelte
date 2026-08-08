<script lang="ts" generics="T, Key = number">
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
		/**
		 * Row keys that must stay mounted even when scrolled outside the visible window
		 * (e.g. the row being dragged in a dnd-kit sortable list). Pinned rows are rendered
		 * as absolutely-positioned copies so their sortable entity keeps a connected element
		 * for the whole drag, which keeps the drag overlay and drop-target tracking working
		 * when the row scrolls out of view.
		 */
		pinnedRowIds?: Key[];
		children: Snippet<[{ item: T; index: number }]>;
	};

	let {
		items,
		height = '100%',
		itemHeight = undefined,
		start = $bindable(0),
		end = $bindable(0),
		rowId,
		pinnedRowIds = [],
		children
	}: Props = $props();

	// local state
	let heightMap: number[] = [];
	let rows: HTMLCollectionOf<HTMLElement>;
	let viewport: HTMLElement;
	let contents: HTMLElement;
	let viewportHeight = $state(0);
	// Rows whose key is in `pinnedRowIds`, regardless of the visible window. They are rendered
	// as absolutely-positioned copies (see the template) so they never unmount while pinned.
	let pinned: { index: number; data: T }[] = $derived.by(() => {
		if (pinnedRowIds.length === 0) return [];

		const result: { index: number; data: T }[] = [];
		for (let i = 0; i < items.length; i++) {
			const id = rowId ? rowId(items[i]) : i;
			if (pinnedRowIds.includes(id as Key)) {
				result.push({ index: i, data: items[i] });
			}
		}
		return result;
	});

	// The visible window. Any pinned row inside the window is replaced by an empty spacer so it
	// keeps occupying its slot in the flow layout (its actual content is rendered absolutely on
	// top of that slot). Spacers are plain divs, so they are ignored by the
	// `svelte-virtual-list-row` collection the scroll math relies on.
	type VisibleRow = { kind: 'row'; index: number; data: T } | { kind: 'spacer'; index: number };
	let visible: VisibleRow[] = $derived.by(() => {
		const rows: VisibleRow[] = [];

		// `start`/`end` are recomputed asynchronously and can lag behind `items` when
		// the list is replaced with a shorter one mid-scroll or mid-drag. Clamp the
		// window to the actual array bounds and skip missing rows so the children
		// snippet is never rendered with an undefined item.
		const first = Math.min(Math.max(start, 0), items.length);
		const last = Math.min(end, items.length);

		for (let i = first; i < last; i++) {
			const data = items[i];
			if (data === undefined) continue;

			if (pinnedRowIds.length === 0) {
				rows.push({ kind: 'row', index: i, data });
				continue;
			}

			const id = rowId ? rowId(data) : i;
			if (pinnedRowIds.includes(id as Key)) {
				rows.push({ kind: 'spacer', index: i });
			} else {
				rows.push({ kind: 'row', index: i, data });
			}
		}

		return rows;
	});

	function rowKey(row: VisibleRow) {
		return row.kind === 'spacer'
			? `__dnd-pinned-spacer-${row.index}`
			: (rowId?.(row.data) ?? row.index);
	}

	// Content-space offset of the row at `index`, used to position pinned rows absolutely within
	// the contents element (which has `position: relative` and `padding-top: top`).
	function contentOffset(index: number) {
		let offset = 0;
		for (let i = 0; i < index; i++) {
			offset += heightMap[i] || itemHeight || average_height || 0;
		}
		return offset;
	}

	let mounted: boolean = $state(false);

	let top = $state(0);
	let bottom = $state(0);
	let average_height: number;

	async function refresh(items: T[], viewportHeight: number, itemHeight: number | undefined) {
		await tick(); // wait until the DOM is up to date

		// If the list shrank below the current scroll window, `start` can point past
		// the new array's end, leaving `end`/`bottom` stale. Reset the window so the
		// last remaining row is shown and it gets recomputed from scratch.
		if (start > items.length) {
			start = Math.max(0, items.length - 1);
			top = 0;
			viewport.scrollTop = 0;
		}

		const { scrollTop } = viewport;

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
	$effect.pre(() => {
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
		style="position: relative; padding-top: {top}px; padding-bottom: {bottom}px;"
	>
		{#each visible as row (rowKey(row))}
			{#if row.kind === 'spacer'}
				<div style="height: {(itemHeight ?? heightMap[row.index]) || 0}px;"></div>
			{:else}
				<svelte-virtual-list-row>
					{@render children({ item: row.data, index: row.index })}
				</svelte-virtual-list-row>
			{/if}
		{/each}

		{#each pinned as row (rowId?.(row.data) ?? row.index)}
			<div style="position: absolute; top: {contentOffset(row.index)}px; left: 0; right: 0;">
				{#if children}{@render children({ item: row.data, index: row.index })}{:else}Missing
					template{/if}
			</div>
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
</style>
