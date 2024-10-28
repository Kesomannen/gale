<script lang="ts">
	import { Menubar } from 'bits-ui';

	export let label: string;
</script>

<Menubar.Menu>
	<Menubar.Trigger
		class="mx-[-1px] h-full cursor-default rounded px-3 text-sm text-gray-400 hover:bg-gray-700 hover:text-gray-200"
	>
		{label}
	</Menubar.Trigger>
	<Menubar.Content
		class="mt-0.5 flex flex-col gap-0.5 rounded-lg border border-gray-600 bg-gray-800 py-1 shadow-xl"
	>
		<slot />
	</Menubar.Content>
</Menubar.Menu>
