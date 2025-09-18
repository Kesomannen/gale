<script lang="ts">
	import Checkbox from '$lib/components/ui/Checkbox.svelte';
	import Info from '$lib/components/ui/Info.svelte';
	import InputField from '$lib/components/ui/InputField.svelte';
	import Label from '$lib/components/ui/Label.svelte';
	import Icon from '@iconify/svelte';

	type Props = {
		value: string[];
		enabled: boolean;
		setValue: (value: string[]) => Promise<void>;
		setEnabled: (enabled: boolean) => Promise<void>;
	};

	let { value = $bindable(), enabled = $bindable(), setValue, setEnabled }: Props = $props();

	let newArg = $state('');
</script>

<div class="mt-1 flex items-center">
	<Label>Set custom launch arguments</Label>

	<Info>
		<p>
			Allows you to add custom arguments to the launch command. Depending on <b>Launch mode</b>,
			these are either ran against the game or launcher executable.
		</p>

		<p>
			Only pass one argument per entry, so instead of <code>--foo value</code>, pass
			<code>--foo</code>
			and <code>value</code> separately.
		</p>
	</Info>

	<Checkbox
		checked={enabled}
		onCheckedChange={(checked) => {
			setEnabled(checked);
		}}
	/>
</div>

{#if enabled}
	<div class="text-primary-300 mt-1 flex flex-col gap-1 pl-[35%]">
		{#each value as argument, i}
			<div class="flex gap-1">
				<button
					class="text-primary-400 hover:bg-primary-700 hover:text-primary-300 rounded-lg p-1.5 text-xl"
					onclick={() => {
						value = value.filter((_, index) => index !== i);
						setValue(value);
					}}
				>
					<Icon icon="mdi:remove" />
				</button>
				<InputField
					class="grow"
					value={argument}
					onchange={(newValue) => {
						value[i] = newValue;
						setValue([...value]);
					}}
				/>
			</div>
		{/each}

		<InputField
			placeholder="Enter new argument..."
			bind:value={newArg}
			onchange={() => {
				if (newArg.length === 0) return;

				value = [...value, newArg];
				newArg = '';
				setValue(value);
			}}
		/>
	</div>
{/if}
