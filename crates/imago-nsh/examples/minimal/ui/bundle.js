import { append, bindProp, component, element, insert, prop, setup, text } from "nsh:rt";
import { effect, mount, signal } from "nsh";
import "nsh/keymap";
import { Window, titleBar } from "nsh/desktop";

//#region ../../../ui/types.ts
const spacing = {
	sidebarWidth: 240,
	titlebarHeight: 40,
	rowHeight: 32,
	compactRowHeight: 28,
	pageGutter: 28
};
const typography = {
	body: 13,
	label: 12,
	title: 24,
	titlebar: 13
};

//#endregion
//#region ../../../ui/icons.ts
const icons = {
	about: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M16%202V6M8%202V6%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M13%204H11C7.22876%204%205.34315%204%204.17157%205.17157C3%206.34315%203%208.22876%203%2012V14C3%2017.7712%203%2019.6569%204.17157%2020.8284C5.34315%2022%207.22876%2022%2011%2022H13C16.7712%2022%2018.6569%2022%2019.8284%2020.8284C21%2019.6569%2021%2017.7712%2021%2014V12C21%208.22876%2021%206.34315%2019.8284%205.17157C18.6569%204%2016.7712%204%2013%204Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M3%2010H21%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M10%2018.5002L9.99999%2013.8474C9.99999%2013.6557%209.86325%2013.5002%209.69458%2013.5002H9M14%2018.4983L15.4855%2013.8923C15.4951%2013.8626%2015.5%2013.8315%2015.5%2013.8002C15.5%2013.6346%2015.3657%2013.5002%2015.2%2013.5002L13%2013.5%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	check: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M5%2013.2592L7.58583%2015.9568C8.2525%2016.6523%208.58583%2017%209.00004%2017C9.41425%2017%209.74759%2016.6523%2010.4143%2015.9568L19%207%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	inbox: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M2.5%2012C2.5%207.52166%202.5%205.28249%203.89124%203.89124C5.28249%202.5%207.52166%202.5%2012%202.5C16.4783%202.5%2018.7175%202.5%2020.1088%203.89124C21.5%205.28249%2021.5%207.52166%2021.5%2012C21.5%2016.4783%2021.5%2018.7175%2020.1088%2020.1088C18.7175%2021.5%2016.4783%2021.5%2012%2021.5C7.52166%2021.5%205.28249%2021.5%203.89124%2020.1088C2.5%2018.7175%202.5%2016.4783%202.5%2012Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M21.5%2013.5H16.5743C15.7322%2013.5%2015.0706%2014.2036%2014.6995%2014.9472C14.2963%2015.7551%2013.4889%2016.5%2012%2016.5C10.5111%2016.5%209.70373%2015.7551%209.30054%2014.9472C8.92942%2014.2036%208.26777%2013.5%207.42566%2013.5H2.5%22%20stroke%3D%22currentColor%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	logbook: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M7.99978%203.5H6.60021C4.43183%203.5%203.34764%203.5%202.67399%204.17362C2.00034%204.84724%202.00029%205.93144%202.00021%208.09982L2%2013.3998C1.99992%2015.5684%201.99987%2016.6526%202.67353%2017.3263C3.34719%2018%204.43146%2018%206.6%2018H8.95042C10.4329%2018%2011.7092%2019.0464%2011.9999%2020.5V5.5C11.0556%204.24097%209.99989%203.5%207.99978%203.5Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M16.0001%203.5H17.3997C19.5681%203.5%2020.6523%203.5%2021.3259%204.17362C21.9996%204.84724%2021.9996%205.93144%2021.9997%208.09982L21.9999%2013.3998C22%2015.5684%2022%2016.6526%2021.3264%2017.3263C20.6527%2018%2019.5684%2018%2017.3999%2018H15.0495C13.567%2018%2012.2907%2019.0464%2012%2020.5V5.5C12.9443%204.24097%2014%203.5%2016.0001%203.5Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	more: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M21%2012C21%2011.1716%2020.3284%2010.5%2019.5%2010.5C18.6716%2010.5%2018%2011.1716%2018%2012C18%2012.8284%2018.6716%2013.5%2019.5%2013.5C20.3284%2013.5%2021%2012.8284%2021%2012Z%22%20stroke%3D%22currentColor%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M13.5%2012C13.5%2011.1716%2012.8284%2010.5%2012%2010.5C11.1716%2010.5%2010.5%2011.1716%2010.5%2012C10.5%2012.8284%2011.1716%2013.5%2012%2013.5C12.8284%2013.5%2013.5%2012.8284%2013.5%2012Z%22%20stroke%3D%22currentColor%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M6%2012C6%2011.1716%205.32843%2010.5%204.5%2010.5C3.67157%2010.5%203%2011.1716%203%2012C3%2012.8284%203.67157%2013.5%204.5%2013.5C5.32843%2013.5%206%2012.8284%206%2012Z%22%20stroke%3D%22currentColor%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	nextWeek: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M16%202V6M8%202V6%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M13%204H11C7.22876%204%205.34315%204%204.17157%205.17157C3%206.34315%203%208.22876%203%2012V14C3%2017.7712%203%2019.6569%204.17157%2020.8284C5.34315%2022%207.22876%2022%2011%2022H13C16.7712%2022%2018.6569%2022%2019.8284%2020.8284C21%2019.6569%2021%2017.7712%2021%2014V12C21%208.22876%2021%206.34315%2019.8284%205.17157C18.6569%204%2016.7712%204%2013%204Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M3%2010H21%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M11%2014H16M8%2014H8.00898M13%2018H8M16%2018H15.991%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	plus: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M12%204V20M20%2012H4%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	project: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M8%207H16.75C18.8567%207%2019.91%207%2020.6667%207.50559C20.9943%207.72447%2021.2755%208.00572%2021.4944%208.33329C22%209.08996%2022%2010.1433%2022%2012.25C22%2015.7612%2022%2017.5167%2021.1573%2018.7779C20.7926%2019.3238%2020.3238%2019.7926%2019.7779%2020.1573C18.5167%2021%2016.7612%2021%2013.25%2021H12C7.28595%2021%204.92893%2021%203.46447%2019.5355C2%2018.0711%202%2015.714%202%2011V7.94427C2%206.1278%202%205.21956%202.38032%204.53806C2.65142%204.05227%203.05227%203.65142%203.53806%203.38032C4.21956%203%205.1278%203%206.94427%203C8.10802%203%208.6899%203%209.19926%203.19101C10.3622%203.62712%2010.8418%204.68358%2011.3666%205.73313L12%207%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	settings: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M3.99963%205.00055L9.99963%205.00031%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M12.9996%205.00031L19.9996%205.00031%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M15.9996%209.00031L15.9996%2015.0003%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M9.99963%202.00031L9.99963%208.00031%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M11.9996%2016.0003L11.9996%2022.0003%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M15.9996%2012.0001L19.9996%2012.0003%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M3.99963%2012.0005L12.9996%2012.0003%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M11.9996%2019.0003L19.9996%2019.0003%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M3.99963%2019.0005L8.99963%2019.0003%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	sidebar: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M2%2012C2%208.31087%202%206.4663%202.81382%205.15877C3.1149%204.67502%203.48891%204.25427%203.91891%203.91554C5.08116%203%206.72077%203%2010%203H14C17.2792%203%2018.9188%203%2020.0811%203.91554C20.5111%204.25427%2020.8851%204.67502%2021.1862%205.15877C22%206.4663%2022%208.31087%2022%2012C22%2015.6891%2022%2017.5337%2021.1862%2018.8412C20.8851%2019.325%2020.5111%2019.7457%2020.0811%2020.0845C18.9188%2021%2017.2792%2021%2014%2021H10C6.72077%2021%205.08116%2021%203.91891%2020.0845C3.48891%2019.7457%203.1149%2019.325%202.81382%2018.8412C2%2017.5337%202%2015.6891%202%2012Z%22%20stroke%3D%22currentColor%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M9.5%203L9.5%2021%22%20stroke%3D%22currentColor%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M5%207H6M5%2010H6%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	someday: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Ccircle%20cx%3D%2212%22%20cy%3D%2212%22%20r%3D%2210%22%20stroke%3D%22currentColor%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M12%208V12L14%2014%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	statistics: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M7%2017L7%2013%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M12%2017L12%207%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M17%2017L17%2011%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M2.5%2012C2.5%207.52166%202.5%205.28249%203.89124%203.89124C5.28249%202.5%207.52166%202.5%2012%202.5C16.4783%202.5%2018.7175%202.5%2020.1088%203.89124C21.5%205.28249%2021.5%207.52166%2021.5%2012C21.5%2016.4783%2021.5%2018.7175%2020.1088%2020.1088C18.7175%2021.5%2016.4783%2021.5%2012%2021.5C7.52166%2021.5%205.28249%2021.5%203.89124%2020.1088C2.5%2018.7175%202.5%2016.4783%202.5%2012Z%22%20stroke%3D%22currentColor%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	trash: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M19.5%205.5L18.8803%2015.5251C18.7219%2018.0864%2018.6428%2019.3671%2018.0008%2020.2879C17.6833%2020.7431%2017.2747%2021.1273%2016.8007%2021.416C15.8421%2022%2014.559%2022%2011.9927%2022C9.42312%2022%208.1383%2022%207.17905%2021.4149C6.7048%2021.1257%206.296%2020.7408%205.97868%2020.2848C5.33688%2019.3626%205.25945%2018.0801%205.10461%2015.5152L4.5%205.5%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M3%205.5H21M16.0557%205.5L15.3731%204.09173C14.9196%203.15626%2014.6928%202.68852%2014.3017%202.39681C14.215%202.3321%2014.1231%202.27454%2014.027%202.2247C13.5939%202%2013.0741%202%2012.0345%202C10.9688%202%2010.436%202%209.99568%202.23412C9.8981%202.28601%209.80498%202.3459%209.71729%202.41317C9.32164%202.7167%209.10063%203.20155%208.65861%204.17126L8.05292%205.5%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M9.5%2016.5L9.5%2010.5%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M14.5%2016.5L14.5%2010.5%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	week: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M16%202V6M8%202V6%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M13%204H11C7.22876%204%205.34315%204%204.17157%205.17157C3%206.34315%203%208.22876%203%2012V14C3%2017.7712%203%2019.6569%204.17157%2020.8284C5.34315%2022%207.22876%2022%2011%2022H13C16.7712%2022%2018.6569%2022%2019.8284%2020.8284C21%2019.6569%2021%2017.7712%2021%2014V12C21%208.22876%2021%206.34315%2019.8284%205.17157C18.6569%204%2016.7712%204%2013%204Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M3%2010H21%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M10%2018.5002L9.99999%2013.8474C9.99999%2013.6557%209.86325%2013.5002%209.69458%2013.5002H9M14%2018.4983L15.4855%2013.8923C15.4951%2013.8626%2015.5%2013.8315%2015.5%2013.8002C15.5%2013.6346%2015.3657%2013.5002%2015.2%2013.5002L13%2013.5%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E",
	display: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M4%2011L4%2021%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M19%2013L19%2021%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M19%203L19%207%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M11.5%203L11.5%2013%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M4%203L4%205%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M11.5%2019L11.5%2021%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M2%209.5C2%209.03406%202%208.80109%202.07612%208.61732C2.17761%208.37229%202.37229%208.17761%202.61732%208.07612C2.80109%208%203.03406%208%203.5%208H4.5C4.96594%208%205.19891%208%205.38268%208.07612C5.62771%208.17761%205.82239%208.37229%205.92388%208.61732C6%208.80109%206%209.03406%206%209.5C6%209.96594%206%2010.1989%205.92388%2010.3827C5.82239%2010.6277%205.62771%2010.8224%205.38268%2010.9239C5.19891%2011%204.96594%2011%204.5%2011H3.5C3.03406%2011%202.80109%2011%202.61732%2010.9239C2.37229%2010.8224%202.17761%2010.6277%202.07612%2010.3827C2%2010.1989%202%209.96594%202%209.5Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M17%2011.5C17%2011.0341%2017%2010.8011%2017.0761%2010.6173C17.1776%2010.3723%2017.3723%2010.1776%2017.6173%2010.0761C17.8011%2010%2018.0341%2010%2018.5%2010H19.5C19.9659%2010%2020.1989%2010%2020.3827%2010.0761C20.6277%2010.1776%2020.8224%2010.3723%2020.9239%2010.6173C21%2010.8011%2021%2011.0341%2021%2011.5C21%2011.9659%2021%2012.1989%2020.9239%2012.3827C20.8224%2012.6277%2020.6277%2012.8224%2020.3827%2012.9239C20.1989%2013%2019.9659%2013%2019.5%2013H18.5C18.0341%2013%2017.8011%2013%2017.6173%2012.9239C17.3723%2012.8224%2017.1776%2012.6277%2017.0761%2012.3827C17%2012.1989%2017%2011.9659%2017%2011.5Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3Cpath%20d%3D%22M9.5%2014.5C9.5%2014.0341%209.5%2013.8011%209.57612%2013.6173C9.67761%2013.3723%209.87229%2013.1776%2010.1173%2013.0761C10.3011%2013%2010.5341%2013%2011%2013H12C12.4659%2013%2012.6989%2013%2012.8827%2013.0761C13.1277%2013.1776%2013.3224%2013.3723%2013.4239%2013.6173C13.5%2013.8011%2013.5%2014.0341%2013.5%2014.5C13.5%2014.9659%2013.5%2015.1989%2013.4239%2015.3827C13.3224%2015.6277%2013.1277%2015.8224%2012.8827%2015.9239C12.6989%2016%2012.4659%2016%2012%2016H11C10.5341%2016%2010.3011%2016%2010.1173%2015.9239C9.87229%2015.8224%209.67761%2015.6277%209.57612%2015.3827C9.5%2015.1989%209.5%2014.9659%209.5%2014.5Z%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-linejoin%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E%0A",
	windowMin: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22M6%2012h12%22%20stroke%3D%22currentColor%22%20stroke-width%3D%222%22%20stroke-linecap%3D%22round%22%2F%3E%3C%2Fsvg%3E%0A",
	windowMax: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Crect%20x%3D%226.5%22%20y%3D%226.5%22%20width%3D%2211%22%20height%3D%2211%22%20rx%3D%221.5%22%20stroke%3D%22currentColor%22%20stroke-width%3D%222%22%2F%3E%3C%2Fsvg%3E%0A",
	windowClose: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%3E%3Cpath%20d%3D%22m6%206%2012%2012M18%206%206%2018%22%20stroke%3D%22currentColor%22%20stroke-linecap%3D%22round%22%20stroke-width%3D%221.5%22%2F%3E%3C%2Fsvg%3E%0A"
};

//#endregion
//#region ../../../ui/components.tsx
function Icon(props) {
	return (() => {
		const __nsh_hv0 = element("image");
		bindProp(__nsh_hv0, "src", () => icons[props.name]);
		bindProp(__nsh_hv0, "style", () => ({
			width: props.size ?? 16,
			height: props.size ?? 16,
			tintColor: props.color
		}));
		return __nsh_hv0;
	})();
}
function NavigationItem(props) {
	return (() => {
		const __nsh_hv1 = element("view");
		bindProp(__nsh_hv1, "onPress", () => props.onPress);
		prop(__nsh_hv1, "role", "button");
		bindProp(__nsh_hv1, "accessibilityState", () => ({ selected: props.selected }));
		prop(__nsh_hv1, "class", "flex-row items-center gap-2 rounded-lg px-2.5");
		bindProp(__nsh_hv1, "style", () => ({
			height: spacing.rowHeight,
			backgroundColor: props.selected ? props.colors.card : "transparent"
		}));
		const __nsh_hv2 = element("text");
		prop(__nsh_hv2, "class", "text-[13px]");
		bindProp(__nsh_hv2, "style", () => ({ color: props.selected ? props.colors.foreground : props.colors.muted }));
		insert(__nsh_hv2, () => props.label);
		append(__nsh_hv1, __nsh_hv2);
		insert(__nsh_hv1, component(Icon, {
			get name() {
				return props.icon;
			},
			get color() {
				return props.colors.muted;
			}
		}), __nsh_hv2);
		return __nsh_hv1;
	})();
}
function Sidebar(props) {
	if (!props.visible) return null;
	const row = (item) => component(NavigationItem, {
		get key() {
			return item.route;
		},
		get label() {
			return item.label;
		},
		get icon() {
			return item.icon;
		},
		get selected() {
			return props.current === item.route;
		},
		get colors() {
			return props.colors;
		},
		onPress: () => props.onNavigate(item.route)
	});
	return (() => {
		const __nsh_hv3 = element("view");
		prop(__nsh_hv3, "class", "shrink-0 border-r px-2 pt-1");
		bindProp(__nsh_hv3, "style", () => ({
			width: spacing.sidebarWidth,
			borderColor: props.colors.border
		}));
		const __nsh_hv4 = element("view");
		prop(__nsh_hv4, "class", "gap-0.5");
		insert(__nsh_hv4, () => props.navigation.map(row));
		const __nsh_hv5 = element("view");
		prop(__nsh_hv5, "class", "mt-5 rounded-lg px-2 pt-2");
		bindProp(__nsh_hv5, "style", () => ({ backgroundColor: props.colors.card }));
		const __nsh_hv6 = element("view");
		prop(__nsh_hv6, "class", "mb-1 h-8 flex-row items-center gap-2 px-0.5");
		const __nsh_hv7 = element("text");
		prop(__nsh_hv7, "class", "flex-1 text-[13px]");
		bindProp(__nsh_hv7, "style", () => ({ color: props.colors.muted }));
		const __nsh_hv8 = text("Проекты");
		append(__nsh_hv7, __nsh_hv8);
		append(__nsh_hv6, __nsh_hv7);
		insert(__nsh_hv6, component(Icon, {
			name: "project",
			get color() {
				return props.colors.muted;
			}
		}), __nsh_hv7);
		insert(__nsh_hv6, component(Icon, {
			name: "plus",
			get color() {
				return props.colors.muted;
			}
		}));
		append(__nsh_hv5, __nsh_hv6);
		insert(__nsh_hv5, () => props.projects.map((project) => (() => {
			const __nsh_hv9 = element("view");
			bindProp(__nsh_hv9, "key", () => project.id);
			prop(__nsh_hv9, "onPress", () => props.onProject(project.id));
			prop(__nsh_hv9, "role", "button");
			prop(__nsh_hv9, "class", "h-8 flex-row items-center gap-2");
			const __nsh_hv10 = element("view");
			prop(__nsh_hv10, "class", "h-2 w-2 rounded-full");
			bindProp(__nsh_hv10, "style", () => ({ backgroundColor: props.colors.muted }));
			const __nsh_hv11 = element("text");
			prop(__nsh_hv11, "class", "text-[13px]");
			bindProp(__nsh_hv11, "style", () => ({ color: props.colors.muted }));
			insert(__nsh_hv11, () => project.title);
			append(__nsh_hv9, __nsh_hv10);
			append(__nsh_hv9, __nsh_hv11);
			return __nsh_hv9;
		})()));
		const __nsh_hv12 = element("view");
		prop(__nsh_hv12, "class", "mt-auto mb-2");
		const __nsh_hv14 = element("view");
		bindProp(__nsh_hv14, "onPress", () => props.onToggleMore);
		prop(__nsh_hv14, "role", "button");
		prop(__nsh_hv14, "class", "h-8 flex-row items-center gap-2 px-2.5");
		const __nsh_hv15 = element("text");
		prop(__nsh_hv15, "class", "text-[13px]");
		bindProp(__nsh_hv15, "style", () => ({ color: props.colors.muted }));
		const __nsh_hv16 = text("Другое");
		append(__nsh_hv15, __nsh_hv16);
		append(__nsh_hv14, __nsh_hv15);
		insert(__nsh_hv14, component(Icon, {
			name: "more",
			get color() {
				return props.colors.muted;
			}
		}), __nsh_hv15);
		append(__nsh_hv12, __nsh_hv14);
		insert(__nsh_hv12, () => props.moreOpen && (() => {
			const __nsh_hv13 = element("view");
			prop(__nsh_hv13, "class", "mb-2 rounded-lg p-1");
			bindProp(__nsh_hv13, "style", () => ({ backgroundColor: props.colors.card }));
			insert(__nsh_hv13, () => props.otherPages.map(row));
			return __nsh_hv13;
		})(), __nsh_hv14);
		append(__nsh_hv3, __nsh_hv4);
		append(__nsh_hv3, __nsh_hv5);
		append(__nsh_hv3, __nsh_hv12);
		return __nsh_hv3;
	})();
}
function Titlebar(props) {
	effect(() => Window.setTitleBar("hidden", spacing.titlebarHeight));
	const controls = titleBar();
	const customControls = controls.left === 0 && controls.right === 0;
	return (() => {
		const __nsh_hv17 = element("view");
		prop(__nsh_hv17, "class", "w-full flex-row items-center");
		bindProp(__nsh_hv17, "style", () => ({ height: controls.height || spacing.titlebarHeight }));
		prop(__nsh_hv17, "desktop", { windowDrag: true });
		const __nsh_hv18 = element("view");
		prop(__nsh_hv18, "class", "flex-row items-center");
		bindProp(__nsh_hv18, "style", () => ({
			width: (props.sidebarVisible === false ? 40 : spacing.sidebarWidth) + controls.left,
			height: spacing.titlebarHeight
		}));
		const __nsh_hv19 = element("view");
		bindProp(__nsh_hv19, "style", () => ({ width: controls.left }));
		const __nsh_hv20 = element("view");
		bindProp(__nsh_hv20, "onPress", () => props.onToggleSidebar);
		prop(__nsh_hv20, "role", "button");
		prop(__nsh_hv20, "accessibilityLabel", "Боковая панель");
		prop(__nsh_hv20, "class", "w-10 items-center justify-center");
		bindProp(__nsh_hv20, "style", () => ({ height: spacing.titlebarHeight }));
		insert(__nsh_hv20, component(Icon, {
			name: "sidebar",
			get color() {
				return props.colors.muted;
			}
		}));
		const __nsh_hv21 = element("view");
		prop(__nsh_hv21, "class", "flex-1");
		prop(__nsh_hv21, "desktop", { windowDrag: true });
		append(__nsh_hv18, __nsh_hv19);
		append(__nsh_hv18, __nsh_hv20);
		append(__nsh_hv18, __nsh_hv21);
		const __nsh_hv25 = element("view");
		bindProp(__nsh_hv25, "onPress", () => props.onAppearance);
		prop(__nsh_hv25, "role", "button");
		prop(__nsh_hv25, "accessibilityLabel", "Настройки отображения");
		prop(__nsh_hv25, "class", "w-10 items-center justify-center");
		bindProp(__nsh_hv25, "style", () => ({ height: spacing.titlebarHeight }));
		insert(__nsh_hv25, component(Icon, {
			name: "display",
			get color() {
				return props.colors.muted;
			}
		}));
		append(__nsh_hv17, __nsh_hv18);
		append(__nsh_hv17, __nsh_hv25);
		insert(__nsh_hv17, () => props.showPageIdentity === false ? (() => {
			const __nsh_hv22 = element("view");
			prop(__nsh_hv22, "class", "flex-1");
			prop(__nsh_hv22, "desktop", { windowDrag: true });
			return __nsh_hv22;
		})() : (() => {
			const __nsh_hv23 = element("view");
			prop(__nsh_hv23, "class", "flex-1 flex-row items-center gap-1.5");
			prop(__nsh_hv23, "desktop", { windowDrag: true });
			const __nsh_hv24 = element("text");
			prop(__nsh_hv24, "class", "font-medium");
			bindProp(__nsh_hv24, "style", () => ({
				fontSize: typography.titlebar,
				color: props.colors.foreground
			}));
			insert(__nsh_hv24, () => props.title);
			append(__nsh_hv23, __nsh_hv24);
			insert(__nsh_hv23, component(Icon, {
				get name() {
					return props.icon;
				},
				size: 15,
				get color() {
					return props.colors.muted;
				}
			}), __nsh_hv24);
			return __nsh_hv23;
		})(), __nsh_hv25);
		insert(__nsh_hv17, () => customControls ? (() => {
			const __nsh_hv26 = element("view");
			prop(__nsh_hv26, "class", "flex-row");
			bindProp(__nsh_hv26, "style", () => ({
				width: 138,
				height: spacing.titlebarHeight
			}));
			const __nsh_hv27 = element("view");
			prop(__nsh_hv27, "onPress", () => Window.minimize());
			prop(__nsh_hv27, "role", "button");
			prop(__nsh_hv27, "accessibilityLabel", "Свернуть окно");
			prop(__nsh_hv27, "class", "w-[46px] items-center justify-center");
			bindProp(__nsh_hv27, "style", () => ({ height: spacing.titlebarHeight }));
			insert(__nsh_hv27, component(Icon, {
				name: "windowMin",
				get color() {
					return props.colors.foreground;
				},
				size: 14
			}));
			const __nsh_hv28 = element("view");
			prop(__nsh_hv28, "onPress", () => Window.toggleMaximize());
			prop(__nsh_hv28, "role", "button");
			prop(__nsh_hv28, "accessibilityLabel", "Развернуть окно");
			prop(__nsh_hv28, "class", "w-[46px] items-center justify-center");
			bindProp(__nsh_hv28, "style", () => ({ height: spacing.titlebarHeight }));
			insert(__nsh_hv28, component(Icon, {
				name: "windowMax",
				get color() {
					return props.colors.foreground;
				},
				size: 14
			}));
			const __nsh_hv29 = element("view");
			prop(__nsh_hv29, "onPress", () => Window.close());
			prop(__nsh_hv29, "role", "button");
			prop(__nsh_hv29, "accessibilityLabel", "Закрыть окно");
			prop(__nsh_hv29, "class", "w-[46px] items-center justify-center");
			bindProp(__nsh_hv29, "style", () => ({ height: spacing.titlebarHeight }));
			insert(__nsh_hv29, component(Icon, {
				name: "windowClose",
				get color() {
					return props.colors.foreground;
				},
				size: 14
			}));
			append(__nsh_hv26, __nsh_hv27);
			append(__nsh_hv26, __nsh_hv28);
			append(__nsh_hv26, __nsh_hv29);
			return __nsh_hv26;
		})() : (() => {
			const __nsh_hv30 = element("view");
			bindProp(__nsh_hv30, "style", () => ({ width: controls.right }));
			return __nsh_hv30;
		})());
		return __nsh_hv17;
	})();
}

//#endregion
//#region App.tsx
setup(null, "{\n  \"cssVars\": {\n    \"theme\": {\n      \"font-sans\": \"Inter, sans-serif\",\n      \"radius\": \"0.625rem\",\n      \"radius-control\": \"0.5rem\",\n      \"spacing-sidebar\": \"240px\",\n      \"spacing-titlebar\": \"40px\",\n      \"spacing-row\": \"32px\",\n      \"font-size-body\": \"13px\",\n      \"font-size-label\": \"12px\"\n    },\n    \"light\": {\n      \"background\": \"#faf9fb\",\n      \"foreground\": \"#26242a\",\n      \"card\": \"#f2f0f4\",\n      \"card-foreground\": \"#26242a\",\n      \"muted\": \"#ece9ef\",\n      \"muted-foreground\": \"#77727e\",\n      \"primary\": \"#7152bf\",\n      \"primary-foreground\": \"#ffffff\",\n      \"border\": \"#dedbe2\",\n      \"input\": \"#dedbe2\",\n      \"ring\": \"#b18aff\"\n    },\n    \"dark\": {\n      \"background\": \"#0d0c10\",\n      \"foreground\": \"#e7e5ea\",\n      \"card\": \"#131216\",\n      \"card-foreground\": \"#e7e5ea\",\n      \"muted\": \"#27262a\",\n      \"muted-foreground\": \"#8d8a94\",\n      \"primary\": \"#af94f9\",\n      \"primary-foreground\": \"#0d0c10\",\n      \"border\": \"#27262a\",\n      \"input\": \"#27262a\",\n      \"ring\": \"#af94f9\"\n    }\n  }\n}\n", null, null);
const colors = {
	background: "#0d0c10",
	foreground: "#e7e5ea",
	muted: "#8d8a94",
	border: "#27262a",
	accent: "#af94f9",
	card: "#131216"
};
const route = signal("home");
function App() {
	const navigation = [{
		route: "home",
		label: "Главная",
		icon: "inbox"
	}];
	return (() => {
		const __nsh_hv0 = element("view");
		prop(__nsh_hv0, "class", "h-full w-full flex-col");
		bindProp(__nsh_hv0, "style", () => ({ backgroundColor: colors.background }));
		const __nsh_hv1 = element("view");
		prop(__nsh_hv1, "class", "flex-1 flex-row");
		const __nsh_hv2 = element("view");
		prop(__nsh_hv2, "class", "flex-1 items-center justify-center");
		const __nsh_hv3 = element("text");
		bindProp(__nsh_hv3, "style", () => ({ color: colors.foreground }));
		const __nsh_hv4 = text("Одна страница приложения");
		append(__nsh_hv3, __nsh_hv4);
		append(__nsh_hv2, __nsh_hv3);
		append(__nsh_hv1, __nsh_hv2);
		insert(__nsh_hv1, component(Sidebar, {
			visible: true,
			get current() {
				return route();
			},
			navigation,
			otherPages: [],
			projects: [],
			moreOpen: false,
			colors,
			get onNavigate() {
				return route.set;
			},
			onProject: () => {},
			onToggleMore: () => {}
		}), __nsh_hv2);
		append(__nsh_hv0, __nsh_hv1);
		insert(__nsh_hv0, component(Titlebar, {
			title: "Пример",
			icon: "about",
			colors,
			onToggleSidebar: () => {},
			onAppearance: () => {}
		}), __nsh_hv1);
		return __nsh_hv0;
	})();
}
mount(App);

//#endregion