<script context="module" lang="ts">
	export let expandedEntry = writable<ConfigEntryId | null>(null);
</script>

<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import ResizableInputField from '$lib/components/ResizableInputField.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import { setConfigEntry } from '$lib/config';
	import type { ConfigEntryId } from '$lib/models';
	import { getListSeparator, type ListSeparator } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Button, Tabs } from 'bits-ui';
	import { writable } from 'svelte/store';
	import { t, T } from '../../i18n';

	let mode: 'text' | 'list' = 'text';
	let newElement = '';
	let separator: ListSeparator = { type: 'default', char: ',' };

	$: open = $expandedEntry !== null;
	$: if (open) reset();

	$: content = ($expandedEntry?.entry.value.content as string) ?? '';
	$: items = content.split(separator.char);

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
</script>

<Popup title="Edit {$expandedEntry?.entry.name}" onClose={() => ($expandedEntry = null)} {open}>
	{#if $expandedEntry !== null && $expandedEntry.entry.value.type === 'string'}
		<TabsMenu
			bind:value={mode}
			options={[
				{
					label: t["Edit as text"],
					value: 'text'
				},
				{
					label: t["Edit as list"],
					value: 'list'
				}
			]}
		>
			<Tabs.Content value="text" class="pt-1">
				<ResizableInputField
					value={content}
					on:change={(e) => {
						content = e.target?.value;
						submitValue();
					}}
				/>
			</Tabs.Content>

			<Tabs.Content value="list" class="pt-1">
				<div class="flex flex-col text-slate-300 gap-1">
					{#each items as element, i}
						<div class="flex gap-1">
							<InputField
								class="flex-grow"
								value={element}
								on:change={({ detail }) => {
									items[i] = detail;
									updateListContent();
								}}
							/>
							<Button.Root
								class="text-slate-400 text-xl hover:text-slate-300 hover:bg-gray-700 p-1.5 rounded-lg"
								on:click={() => {
									items.splice(i, 1);
									updateListContent();
								}}
							>
								<Icon icon="mdi:remove" />
							</Button.Root>
						</div>
					{/each}

					<InputField
						class="pr-9 mt-1 w-full"
						placeholder= {t["Enter new value"]}
						bind:value={newElement}
						on:change={() => {
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
