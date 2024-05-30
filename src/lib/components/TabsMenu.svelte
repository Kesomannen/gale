<script lang="ts">
  import { Tabs } from "bits-ui";

  export let value: string;
  export let options: { value: string, label: string }[];
</script>

<Tabs.Root bind:value>
  <Tabs.List class="flex gap-1 p-1 my-1 rounded-xl text-slate-300 bg-gray-900">
    {#each options as option}
      <Tabs.Trigger
        value={option.value}
        class="flex-grow rounded-lg px-2 py-0.5
              hover:bg-gray-800 hover:text-slate-100
              data-[state=active]:bg-gray-700 data-[state=active]:text-slate-100 data-[state=active]:font-semibold"
      >
        {option.label}
      </Tabs.Trigger>
    {/each}
  </Tabs.List>
    
  <slot />
</Tabs.Root>