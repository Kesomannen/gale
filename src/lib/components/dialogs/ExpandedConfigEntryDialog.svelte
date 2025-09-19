<script lang="ts">
	import InputField from '$lib/components/ui/InputField.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import ResizableInputField from '$lib/components/ui/ResizableInputField.svelte';
	import TabsMenu from '$lib/components/ui/TabsMenu.svelte';
	import { setConfigEntry } from '$lib/config';
	import { getListSeparator, type ListSeparator } from '$lib/util';
	import Icon from '@iconify/svelte';
	import { Tabs } from 'bits-ui';
	import { config } from '$lib/state/misc.svelte';
	import { expandedConfigEntryDialog_title, m } from '$lib/paraglide/messages';

	let mode: 'text' | 'list' = $state('text');
	let newElement = $state('');
	let separator: ListSeparator = $state({ type: 'default', char: ',' });

	let previousOpen = false;

	async function updateListContent() {
		content = items.join(separator.char);
		await submitValue();
	}

	async function submitValue() {
		if (!config.expandedEntry) return;

		await setConfigEntry(config.expandedEntry, {
			type: 'string',
			content
		});

		config.expandedEntry.entry.value.content = content;
	}

	function reset() {
		if (!config.expandedEntry || previousOpen) return;
		previousOpen = true;

		mode = 'text';
		newElement = '';

		content = config.expandedEntry.entry.value.content as string;
		separator = getListSeparator(config.expandedEntry.entry);
	}

	let open = $derived(config.expandedEntry !== null);

	let content = $derived((config.expandedEntry?.entry.value.content as string) ?? '');
	let items = $derived(content.split(separator.char));

	$effect(() => {
		if (open) {
			reset();
		} else {
			previousOpen = false;
		}
	});
</script>

<Dialog
	large
	title={m.expandedConfigEntryDialog_title({ name : config.expandedEntry?.entry.name ?? m.unknown() })}
	onclose={() => (config.expandedEntry = null)}
	{open}
>
	{#if config.expandedEntry && config.expandedEntry.entry.value.type === 'string'}
		<TabsMenu
			bind:value={mode}
			options={[
				{
					label: m.expandedConfigEntryDialog_tabsMenu_text(),
					value: 'text'
				},
				{
					label: m.expandedConfigEntryDialog_tabsMenu_list(),
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
						placeholder={m.expandedConfigEntryDialog_placeholder()}
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
</Dialog>
