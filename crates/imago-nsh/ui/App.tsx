import { mount, Text, View } from "nsh";
import { Icon, Titlebar } from "./index";

function App() {
  const colors = { background: "#0d0c10", foreground: "#e7e5ea", muted: "#8d8a94", border: "#27262a", accent: "#af94f9", card: "#131216" };
  return <View class="h-full w-full" style={{ backgroundColor: colors.background }}>
    <Titlebar title="Imago" icon="about" colors={colors} onToggleSidebar={() => {}} onAppearance={() => {}} />
    <View class="flex-1 items-center justify-center">
      <Icon name="week" size={24} color={colors.accent} />
      <Text style={{ color: colors.foreground }}>Shared NSH components</Text>
    </View>
  </View>;
}

mount(App);
