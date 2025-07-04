<script module lang="ts">
	export let expandedEntry = writable<ConfigEntryId | null>(null);
</script>

<script lang="ts">
	import InputField from '$lib/components/ui/InputField.svelte';
	import Popup from '$lib/components/ui/Popup.svelte';
	import ResizableInputField from '$lib/components/ui/ResizableInputField.svelte';
	import TabsMenu from '$lib/components/ui/TabsMenu.svelte';
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId } from '$lib/types';
	import { getListSeparator, type ListSeparator } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';
	import { writable } from 'svelte/store';

	let mode: 'text' | 'list' = $state('text');
	let newElement = $state('');
	let separator: ListSeparator = $state({ type: 'default', char: ',' });

	async function updateListContent() {
		content = items.join(separator.char);
		await submitValue();
	}

	async function submitValue() {
		if ($expandedEntry === null) return;

		await setConfigEntry($expandedEntry, {
			type: 'string',
			content
		});

		$expandedEntry.entry.value.content = content;
	}

	function reset() {
		if ($expandedEntry === null) return;

		mode = 'text';
		newElement = '';

		content = $expandedEntry.entry.value.content as string;
		separator = getListSeparator($expandedEntry.entry);
	}

	let open = $derived($expandedEntry !== null);

	let content = $derived(($expandedEntry?.entry.value.content as string) ?? '');
	let items = $derived(content.split(separator.char));

	$effect(() => {
		if (open) reset();
	});
</script>

<Popup
	large
	title="Edit {$expandedEntry?.entry.name}"
	onclose={() => ($expandedEntry = null)}
	{open}
>
	{#if $expandedEntry !== null && $expandedEntry.entry.value.type === 'string'}
		<TabsMenu
			bind:value={mode}
			options={[
				{
					label: 'Edit as text',
					value: 'text'
				},
				{
					label: 'Edit as list',
					value: 'list'
				}
			]}
		>
			<Tabs.Content value="text" class="pt-1">
				<ResizableInputField
					value={content}
					onchange={(evt) => {
						content = evt.currentTarget.value;
						submitValue();
					}}
				/>
			</Tabs.Content>

			<Tabs.Content value="list" class="pt-1">
				<div class="text-primary-300 flex flex-col gap-1">
					{#each items as element, i}
						<div class="flex gap-1">
							<button
								class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 rounded-lg p-1.5 text-xl"
								onclick={() => {
									items.splice(i, 1);
									updateListContent();
								}}
							>
								<Icon icon="mdi:remove" />
							</button>
							<InputField
								class="grow"
								value={element}
								onchange={(value) => {
									items[i] = value;
									updateListContent();
								}}
							/>
						</div>
					{/each}

					<InputField
						class="mt-1 w-full pr-9"
						placeholder="Enter new value..."
						bind:value={newElement}
						onchange={() => {
							if (newElement.length === 0) return;

							items.push(newElement);
							newElement = '';
							updateListContent();
						}}
					/>
				</div>
			</Tabs.Content>
		</TabsMenu>
	{/if}
</Popup>
