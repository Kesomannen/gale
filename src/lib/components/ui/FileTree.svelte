<script lang="ts" generics="T">
	import type { ClassValue } from 'clsx';
	import Icon from '@iconify/svelte';
	import Checkbox from './Checkbox.svelte';
	import ChecklistShell from './ChecklistShell.svelte';
	import { shortenFileSize } from '$lib/util';
	import DropdownArrow from './DropdownArrow.svelte';

	type Props = {
		class?: ClassValue;
		items: T[];
		getPath: (item: T) => string;
		getSize?: (item: T) => number;
		get: (item: T) => boolean;
		set: (item: T, value: boolean) => void;
		setMany?: (items: T[], value: boolean) => void;
		title?: string;
		maxHeight?: 'none' | 'sm' | 'md';
	};

	let {
		class: classProp,
		items,
		getPath,
		getSize,
		get,
		set,
		setMany,
		title,
		maxHeight = 'none'
	}: Props = $props();

	type FileEntry = {
		item: T;
		name: string;
		path: string;
	};

	type TreeNode = {
		name: string;
		path: string;
		dirs: TreeNode[];
		files: FileEntry[];
		/**
		 * Every file nested under this node, used to toggle whole directories at once,
		 * as well as determine total directory size and checkbox state.
		 */
		allItems: T[];
	};

	/** A single row in the flattened, rendered list. */
	type Row =
		| { type: 'dir'; node: TreeNode; depth: number; key: string }
		| { type: 'file'; entry: FileEntry; depth: number; key: string };

	/** Normalise separators to `/` and split a path into its directory + file segments. */
	function splitPath(path: string): string[] {
		return path
			.replace(/\\/g, '/')
			.split('/')
			.filter((segment) => segment.length > 0);
	}

	function createNode(name: string, path: string): TreeNode {
		return { name, path, dirs: [], files: [], allItems: [] };
	}

	/** Insert one file into the tree, creating any missing directory nodes along the way. */
	function insertEntry(tree: TreeNode, item: T, segments: string[]): void {
		const name = segments[segments.length - 1] ?? '';
		const path = segments.join('/');

		// descend down the tree, creating any missing nodes as we go
		let node = tree;
		let nodePath = '';
		for (const segment of segments.slice(0, -1)) {
			nodePath = nodePath ? `${nodePath}/${segment}` : segment;

			let child = node.dirs.find((dir) => dir.name === segment);
			if (!child) {
				// directory doesn't exist yet, create it
				child = createNode(segment, nodePath);
				node.dirs.push(child);
			}
			node = child;
		}

		// the final segment is the file leaf node
		node.files.push({ item, name, path });
	}

	/** Sort a node's children and fill in its aggregated list of descendant files. */
	function finalizeNode(node: TreeNode): TreeNode {
		node.dirs.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));
		node.files.sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }));

		node.allItems = [
			...node.files.map((entry) => entry.item),
			...node.dirs.flatMap((dir) => finalizeNode(dir).allItems)
		];

		return node;
	}

	/** The root of the file tree. The root node itself represents the profile directory. */
	const root = $derived.by(() => {
		const tree = createNode('', '');

		for (const item of items) {
			insertEntry(tree, item, splitPath(getPath(item)));
		}

		return finalizeNode(tree);
	});

	let expanded = $state<Record<string, boolean>>({});

	function toggleExpand(path: string) {
		expanded[path] = !expanded[path];
	}

	/** Flatten the tree into the rows currently visible. */
	function flattenRows(root: TreeNode, expanded: Record<string, boolean>): Row[] {
		const rows: Row[] = [];

		const visit = (node: TreeNode, depth: number) => {
			for (const dir of node.dirs) {
				rows.push({ type: 'dir', node: dir, depth, key: `dir:${dir.path}` });
				// only visit expanded directories
				if (expanded[dir.path]) visit(dir, depth + 1);
			}

			for (const entry of node.files) {
				rows.push({ type: 'file', entry, depth, key: `file:${entry.path}` });
			}
		};

		visit(root, 0);

		return rows;
	}

	const rows = $derived.by(() => flattenRows(root, expanded));

	function dirState(node: TreeNode): 'all' | 'some' | 'none' {
		let all = true;
		let any = false;

		for (const item of node.allItems) {
			if (get(item)) any = true;
			else all = false;
		}

		return all ? 'all' : any ? 'some' : 'none';
	}

	function totalSize(node: TreeNode): number {
		let sum = 0;
		for (const item of node.allItems) sum += getSize?.(item) ?? 0;
		return sum;
	}

	/** Emphasise the size label proportionally to the file/directory size. */
	function sizeClass(size: number): string {
		// 100 MiB
		if (size >= 100 * 1024 * 1024) return 'font-semibold text-primary-800 dark:text-primary-100';
		// 10 MiB
		if (size >= 10 * 1024 * 1024) return 'font-medium text-primary-700 dark:text-primary-200';
		// 1 MiB
		if (size >= 1024 * 1024) return 'font-medium text-primary-700 dark:text-primary-200';
		// 100 KiB
		if (size >= 100 * 1024) return 'text-primary-600 dark:text-primary-300';
		return 'text-primary-500 dark:text-primary-400';
	}

	function apply(itemsToSet: T[], value: boolean) {
		if (setMany) setMany(itemsToSet, value);
		else itemsToSet.forEach((item) => set(item, value));
	}

	function toggleDir(node: TreeNode, value: boolean) {
		apply(node.allItems, value);
	}

	function toggleAll(value: boolean) {
		apply(items, value);
	}

	const allIncluded = $derived(items.length > 0 && items.every(get));
	const anyIncluded = $derived(items.some(get));
</script>

{#snippet sizeLabel(size: number)}
	<span class={['ml-auto shrink-0 pl-3 text-sm', sizeClass(size)]}>
		{shortenFileSize(size)}
	</span>
{/snippet}

<ChecklistShell {title} {allIncluded} {anyIncluded} onToggleAll={toggleAll} class={classProp}>
	<div
		class={[
			'min-h-0 flex-1 overflow-y-auto',
			maxHeight === 'md' && 'max-h-120',
			maxHeight === 'sm' && 'max-h-80'
		]}
	>
		{#each rows as row (row.key)}
			{@const rowClass =
				'text-primary-700 dark:text-primary-300 dark:even:bg-primary-900/30 even:bg-primary-100 flex items-center py-2 pr-4'}
			{@const rowStyle = `padding-left: calc(var(--spacing) * ${4 + row.depth * 6})`}

			{#if row.type === 'dir'}
				{@const node = row.node}
				{@const state = dirState(node)}
				{@const open = expanded[node.path]}

				<div class={rowClass} style={rowStyle}>
					<Checkbox
						class="mr-3 shrink-0"
						checked={state === 'all'}
						indeterminate={state === 'some'}
						onCheckedChange={(value) => toggleDir(node, value)}
					/>

					<button class="flex grow items-center gap-1.5" onclick={() => toggleExpand(node.path)}>
						<DropdownArrow {open} />
						<Icon
							icon={open ? 'mdi:folder-open' : 'mdi:folder'}
							class="text-primary-500 dark:text-primary-400 shrink-0 text-xl"
						/>

						<span class="truncate">{node.name}</span>
					</button>

					{#if getSize}
						{@render sizeLabel(totalSize(node))}
					{/if}
				</div>
			{:else}
				{@const entry = row.entry}

				<label class={rowClass} style={rowStyle}>
					<Checkbox
						class="mr-3 shrink-0"
						checked={get(entry.item)}
						onCheckedChange={(value) => set(entry.item, value)}
					/>

					<span class="grow truncate" title={entry.path}>{entry.name}</span>

					{#if getSize}
						{@render sizeLabel(getSize(entry.item))}
					{/if}
				</label>
			{/if}
		{/each}
	</div>
</ChecklistShell>
