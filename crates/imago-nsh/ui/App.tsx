import { mount, signal, Text, View } from "nsh";
import { Icon, Titlebar } from "./index";

function App() {
  const actionCount = signal(0);
  const colors = { background: "#0d0c10", foreground: "#e7e5ea", muted: "#8d8a94", border: "#27262a", accent: "#af94f9", card: "#131216" };
  return <View class="h-full w-full" style={{ backgroundColor: colors.background }}>
    <Titlebar title="Imago" icon="about" colors={colors} onToggleSidebar={() => {}} onAppearance={() => {}} actions={[{ id: "calendar", label: "Календарь дневника", icon: "sidebar" }]} onAction={() => actionCount.update((count) => count + 1)} />
    <View class="flex-1 items-center justify-center">
      <Icon name="week" size={24} color={colors.accent} />
      <Text style={{ color: colors.foreground }}>Shared NSH components</Text>
      <Text>{actionCount()}</Text>
    </View>
  </View>;
}

mount(App);
