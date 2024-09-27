<script lang="ts">
	import Button, { type Color, type Weight } from '$lib/components/Button.svelte';
	import Select from '$lib/components/Select.svelte';
	import { capitalize } from '$lib/util';

	let selected: string = 'one';
</script>

<div class="space-y-5 p-6">
	<a href="/test/2fdff3bd-6086-4ad8-adea-16374fee4fdc">link</a>

	<div class="flex gap-2 p-8">
		<div class="grid grid-cols-3 gap-2">
			{#each ['green', 'blue', 'red', 'gray'] as color}
				{#each ['primary', 'secondary', 'tertiary'] as weight}
					<Button
						color={color as Color}
						weight={weight as Weight}
						label="Install"
						icon="material-symbols:download"
					/>
				{/each}
			{/each}
		</div>
	</div>

	<div class="flex gap-2 p-8">
		<Select
			class="min-w-[200px]"
			icon="material-symbols:download"
			items={['one', 'two', 'three', 'four']}
			getLabel={capitalize}
			bind:selected
		/>

		{selected}
	</div>
</div>
