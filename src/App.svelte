<script lang="ts">
  import { onMount, settled, tick } from "svelte";
  import * as fabric from "fabric";
  import Emoji from "./lib/Emoji.svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Environment detection
  const is_tauri = !!(window as any).__TAURI_INTERNALS__;

  // Constants
  const WORKSPACE_W = 800;
  const WORKSPACE_H = 200;

  // Dimensions in mm
  let label_width_mm = $state(parseFloat(localStorage.getItem("label_width_mm") || "40"));
  let label_height_mm = $state(parseFloat(localStorage.getItem("label_height_mm") || "12"));
  
  // Width and height in 0.125 mm units
  let label_width = $derived(Math.round(label_width_mm * 8));
  let label_height = $derived(Math.round(label_height_mm * 8));
  
  let text = $state(sessionStorage.getItem("text") || "");
  let show_emoji = $state(false);
  let font_size = $state(parseFloat(localStorage.getItem("font_size") || "48"));
  let search_all_devices = $state(localStorage.getItem("search_all_devices") === "true");
  let border_style = $state(localStorage.getItem("border_style") || "none");
  let border_thickness = $state(parseFloat(localStorage.getItem("border_thickness") || "2"));
  let border_margin = $state(parseFloat(localStorage.getItem("border_margin") || "1"));
  
  let is_printing = $state(false);
  let grid_enabled = $state(localStorage.getItem("grid_enabled") === "true");
  let grid_size = $state(parseFloat(localStorage.getItem("grid_size") || "8"));

  // Save to localStorage
  $effect(() => {
    localStorage.setItem("label_width_mm", label_width_mm.toString());
    localStorage.setItem("label_height_mm", label_height_mm.toString());
    localStorage.setItem("font_size", font_size.toString());
    localStorage.setItem("search_all_devices", search_all_devices.toString());
    localStorage.setItem("border_style", border_style);
    localStorage.setItem("border_thickness", border_thickness.toString());
    localStorage.setItem("border_margin", border_margin.toString());
    localStorage.setItem("grid_enabled", grid_enabled.toString());
    localStorage.setItem("grid_size", grid_size.toString());
  });

  // Automatic canvas background and grid update
  $effect(() => {
    if (fabricCanvas && label_width && label_height) {
      renderCanvasBackground();
      addBorder(border_style);
    }
  });

  let canvas: HTMLCanvasElement | undefined = $state();
  let fabricCanvas: fabric.Canvas | undefined = $state();
  let active_object: fabric.FabricObject | undefined = $state();
  let pixelData = new Uint8Array();
  let updateTimeout: number | undefined;
  
  let show_icons = $state(false);
  let icon_search = $state("");
  let is_searching_icons = $state(false);
  let icon_results: Array<{ prefix: string; name: string; url: string; }> = $state([]);

  let show_font_picker = $state(false);
  let font_search = $state("");
  let all_google_fonts: string[] = $state([]);
  let is_loading_fonts = $state(false);
  let font_page = $state(0);
  const FONTS_PER_PAGE = 48;

  const FONTS = ["sans-serif", "serif", "monospace", "cursive", "fantasy", "Arial", "Verdana", "Times New Roman", "Courier New", "Georgia", "Libre Barcode 39", "More fonts..."];

  let searched_fonts = $derived(
    font_search === "" 
      ? all_google_fonts 
      : all_google_fonts.filter(f => f.toLowerCase().includes(font_search.toLowerCase()))
  );

  let total_font_pages = $derived(Math.ceil(searched_fonts.length / FONTS_PER_PAGE));

  let filtered_fonts = $derived(
    searched_fonts.slice(font_page * FONTS_PER_PAGE, (font_page + 1) * FONTS_PER_PAGE)
  );

  async function fetchGoogleFonts() {
    if (all_google_fonts.length > 0) return;
    is_loading_fonts = true;
    
    // 300+ most popular Google Fonts for offline/robust usage
    const BUILT_IN_FONTS = [
      "Roboto", "Open Sans", "Lato", "Montserrat", "Oswald", "Source Sans Pro", "Slabo 27px", "Raleway", "PT Sans", "Merriweather", "Noto Sans", "Arimo", "Playfair Display", "Poppins", "Lora", "Muli", "Amatic SC", "Nunito", "Bebas Neue", "Pacifico", "Dancing Script", "Abril Fatface", "Ubuntu", "Comfortaa", "Exo 2", "Caveat", "Indie Flower", "Inconsolata", "Kanit", "Crimson Text", "Quicksand", "Anton", "Lobster", "Josefin Sans", "Libre Baskerville", "Signika", "Teko", "Domine", "Oxygen", "Archivo", "Titillium Web", "Bitter", "PT Serif", "Dosis", "Fira Sans", "Hind", "Cabin", "Heebo", "Mukta", "Karla", "Spectral", "Prompt", "Work Sans", "Nanum Gothic", "Zilla Slab", "Cinzel", "Righteous", "Shadows Into Light", "Permanent Marker", "Sacramento", "Courgette", "Orbitron", "Kalam", "Gloria Hallelujah", "Satisfy", "Creepster", "Yellowtail", "Bangers", "Cookie", "Kaushan Script", "Patua One", "Lobster Two", "Varela Round", "Questrial", "Old Standard TT", "Gilda Display", "Cormorant Garamond", "Cinzel Decorative", "Julius Sans One", "Playball", "Tangerine", "Great Vibes", "Allura", "Alex Brush", "Rochester", "Arizonia", "Petit Formal Script", "Mrs Saint Delafield", "Pinyon Script", "Monsieur La Doulaise", "League Script", "Herr Von Muellerhoff", "Parisienne", "Italianno", "Engagement", "Grand Hotel", "Rouge Script", "Damion", "Berkshire Swash", "Homemade Apple", "Bad Script", "Marck Script", "Reenie Beanie", "Nothing You Could Do", "Covered By Your Grace", "Waiting for the Sunrise", "Architects Daughter", "Shadows Into Light Two", "Patrick Hand", "Coming Soon", "Just Another Hand", "Rock Salt", "Walter Turncoat", "Schoolbell", "Short Stack", "Indie Flower", "Boogaloo", "Special Elite", "UnifrakturMaguntia", "Metal Mania", "Jim Nightshade", "Eater", "Nosifer", "Butcherman", "Frijole", "Ewert", "Diplomata SC", "Sancreek", "Snowburst One", "Stalinist One", "Barrio", "Finger Paint", "Ribeye Marrow", "Fascinate Inline", "Faster One", "Monoton", "Wallpoet", "Megrim", "Press Start 2P", "Codystar", "Bungee Inline", "Vampiro One", "Monofett", "Erica One", "Chonburi", "Sriracha", "Itim", "Mali", "Mitr", "Sarabun", "Bai Jamjuree", "Chakra Petch", "Krub", "KoHo", "Kodchasan", "Fahkwang", "Taviraj", "Pridi", "Maitree", "Trirong", "Athiti", "Charm", "Charmonman", "Srisakdi", "Niramit", "Kufam", "Jura", "Michroma", "Syncopate", "Nova Mono", "Share Tech Mono", "VT323", "Fira Code", "Space Mono", "JetBrains Mono", "IBM Plex Mono", "Source Code Pro", "Nanum Gothic Coding", "Cutive Mono", "Major Mono Display", "Courier Prime", "Anonymous Pro", "Red Hat Mono", "Overpass Mono", "Roboto Mono", "Ubuntu Mono", "PT Mono", "Oxygen Mono", "Nova Square", "Electrolize", "Russo One", "Audiowide", "Iceberg", "Exo", "Quantico", "Black Ops One", "Stardos Stencil", "Special Elite", "Allerta Stencil", "Keania One", "Big Shoulders Display", "Archivo Black", "Staatliches", "Squada One", "Passion One", "Francois One", "Paytone One", "Chivo", "Alfa Slab One", "Luckiest Guy", "Bowlby One SC", "Carter One", "Rokkitt", "Arvo", "Josefin Slab", "Sanchez", "Crete Round", "Noticia Text", "Vollkorn", "Cardo", "Gentium Basic", "Crimson Pro", "EB Garamond", "Faustina", "Alice", "Abhaya Libre", "Prata"
    ];

    try {
      const response = await fetch("https://gwfh.mranix.com/api/fonts");
      if (!response.ok) throw new Error("API down");
      const data = await response.json();
      all_google_fonts = Array.from(new Set([...BUILT_IN_FONTS, ...data.map((f: any) => f.family)])).sort();
    } catch (err) {
      console.warn("Primary font API failed, using built-in library:", err);
      all_google_fonts = Array.from(new Set(BUILT_IN_FONTS)).sort();
    } finally { is_loading_fonts = false; }
  }

  // Pre-load previews for current page
  $effect(() => {
    if (show_font_picker && filtered_fonts.length > 0) {
      filtered_fonts.forEach(family => {
        const linkId = `preview-${family.replace(/\s+/g, '-').toLowerCase()}`;
        if (!document.getElementById(linkId)) {
          const link = document.createElement('link');
          link.id = linkId; link.rel = 'stylesheet';
          link.href = `https://fonts.googleapis.com/css2?family=${family.replace(/\s+/g, '+')}&text=${encodeURIComponent(family)}&display=swap`;
          document.head.appendChild(link);
        }
      });
    }
  });

  async function loadGoogleFont(family: string) {
    if (!fabricCanvas || !active_object) return;
    const linkId = `font-${family.replace(/\s+/g, '-').toLowerCase()}`;
    if (!document.getElementById(linkId)) {
      const link = document.createElement('link');
      link.id = linkId; link.rel = 'stylesheet';
      link.href = `https://fonts.googleapis.com/css2?family=${family.replace(/\s+/g, '+')}&display=swap`;
      document.head.appendChild(link);
      try { await document.fonts.load(`16px "${family}"`); } catch (e) {}
    }
    formatObject('fontFamily', family);
    // Removed show_font_picker = false; to keep drawer open for rapid testing
  }

  function handleFontChange(val: string) {
    if (val === "More fonts...") { 
      show_font_picker = true; show_icons = false; show_emoji = false; 
      fetchGoogleFonts(); 
    } else { 
      formatObject('fontFamily', val); 
    }
  }

  let templates: Array<{name: string, data: any}> = $state(JSON.parse(localStorage.getItem("label_templates") || "[]"));

  function saveTemplate() {
    if (!fabricCanvas) return;
    const name = prompt("Template Name:");
    if (!name) return;
    templates = [...templates, { name, data: fabricCanvas.toJSON() }];
    localStorage.setItem("label_templates", JSON.stringify(templates));
  }

  function deleteTemplate(index: number) {
    templates = templates.filter((_, i) => i !== index);
    localStorage.setItem("label_templates", JSON.stringify(templates));
  }

  async function loadTemplate(templateData: any) {
    if (!fabricCanvas) return;
    const objects = templateData.objects || [];
    const customFonts = new Set<string>();
    objects.forEach((obj: any) => { if (obj.fontFamily && !FONTS.includes(obj.fontFamily)) customFonts.add(obj.fontFamily); });
    await Promise.all(Array.from(customFonts).map(f => loadGoogleFont(f as string)));
    await fabricCanvas.loadFromJSON(templateData);
    renderCanvasBackground();
    addBorder(border_style);
    fabricCanvas.renderAll();
    debouncedUpdate();
  }

  function renderCanvasBackground() {
    if (!fabricCanvas) return;
    const existing = fabricCanvas.getObjects().filter(obj => (obj as any).data?.isBackground);
    fabricCanvas.remove(...existing);

    // Label area (Centered White)
    const labelBg = new fabric.Rect({
      left: WORKSPACE_W / 2,
      top: WORKSPACE_H / 2,
      originX: 'center',
      originY: 'center',
      width: label_width, 
      height: label_height,
      fill: "#fff", 
      selectable: false, 
      evented: false,
      data: { isBackground: true },
      shadow: new fabric.Shadow({ color: 'rgba(0,0,0,0.3)', blur: 10 })
    });
    fabricCanvas.add(labelBg);

    const l = WORKSPACE_W / 2 - label_width / 2;
    const t = WORKSPACE_H / 2 - label_height / 2;

    if (grid_enabled) {
      for (let i = 0; i <= label_width; i += grid_size) {
        const line = new fabric.Line([l + i, t, l + i, t + label_height], { stroke: "#ddd", selectable: false, evented: false });
        line.set('data', { isBackground: true });
        fabricCanvas.add(line);
      }
      for (let i = 0; i <= label_height; i += grid_size) {
        const line = new fabric.Line([l, t + i, l + label_width, t + i], { stroke: "#ddd", selectable: false, evented: false });
        line.set('data', { isBackground: true });
        fabricCanvas.add(line);
      }
    }
    
    const bgObjects = fabricCanvas.getObjects().filter(o => (o as any).data?.isBackground);
    bgObjects.forEach(o => fabricCanvas?.sendObjectToBack(o));
    fabricCanvas.renderAll();
  }

  function snapToGrid(obj: fabric.FabricObject) {
    if (!grid_enabled) return;
    const l = WORKSPACE_W / 2 - label_width / 2;
    const t = WORKSPACE_H / 2 - label_height / 2;
    const relLeft = obj.left! - l; const relTop = obj.top! - t;
    obj.set({
      left: l + Math.round(relLeft / grid_size) * grid_size,
      top: t + Math.round(relTop / grid_size) * grid_size
    });
  }

  function addBorder(style: string) {
    if (!fabricCanvas) return;
    const existing = fabricCanvas.getObjects().filter(obj => (obj as any).data?.isBorder);
    fabricCanvas.remove(...existing);
    if (style === 'none') { fabricCanvas.renderAll(); debouncedUpdate(); return; }

    const margin = Math.round(border_margin * 8);
    const thickness = style === 'thin' ? 1 : Math.round(border_thickness);
    const w = label_width - (margin * 2) - thickness;
    const h = label_height - (margin * 2) - thickness;

    const common = {
      left: WORKSPACE_W / 2, top: WORKSPACE_H / 2,
      originX: 'center' as any, originY: 'center' as any,
      width: w, height: h, fill: 'transparent', stroke: '#000', strokeWidth: thickness,
      selectable: false, evented: false, strokeUniform: true, data: { isBorder: true }
    };

    let border;
    switch (style) {
      case 'solid': case 'thin': border = new fabric.Rect({ ...common }); break;
      case 'dashed': border = new fabric.Rect({ ...common, strokeDashArray: [10, 5] }); break;
      case 'dotted': border = new fabric.Rect({ ...common, strokeDashArray: [thickness, thickness] }); break;
      case 'rounded': border = new fabric.Rect({ ...common, rx: 10, ry: 10 }); break;
      case 'double':
        const outer = new fabric.Rect({ ...common });
        const inner = new fabric.Rect({ ...common, width: w - thickness * 4, height: h - thickness * 4, strokeWidth: Math.max(1, thickness / 2) });
        border = new fabric.Group([outer, inner], { selectable: false, evented: false, originX: 'center', originY: 'center', left: WORKSPACE_W / 2, top: WORKSPACE_H / 2 });
        border.set('data', { isBorder: true });
        break;
      case 'brackets':
        const len = Math.min(w, h) * 0.3;
        border = new fabric.Path(`M ${len} 0 L 0 0 L 0 ${len} M 0 ${h-len} L 0 ${h} L ${len} ${h} M ${w-len} ${h} L ${w} ${h} L ${w} ${h-len} M ${w} ${len} L ${w} 0 L ${w-len} 0`, { ...common, width: undefined, height: undefined, fill: '' });
        break;
    }

    if (border) {
      fabricCanvas.add(border); fabricCanvas.sendObjectToBack(border);
      const bgObjects = fabricCanvas.getObjects().filter(o => (o as any).data?.isBackground);
      bgObjects.forEach(o => fabricCanvas?.sendObjectToBack(o));
      fabricCanvas.renderAll(); debouncedUpdate();
    }
  }

  function addShape(type: string) {
    if (!fabricCanvas) return;
    let shape;
    const common = { left: WORKSPACE_W / 2, top: WORKSPACE_H / 2, fill: "transparent", stroke: "#000", strokeWidth: 2, originX: "center" as any, originY: "center" as any };
    switch (type) {
      case 'rect': shape = new fabric.Rect({ ...common, width: 50, height: 30 }); break;
      case 'circle': shape = new fabric.Circle({ ...common, radius: 25 }); break;
      case 'line': shape = new fabric.Line([0, 0, 100, 0], { ...common, fill: '#000' }); break;
    }
    if (shape) { fabricCanvas.add(shape); fabricCanvas.setActiveObject(shape); fabricCanvas.renderAll(); debouncedUpdate(); }
  }

  function createRulerGroup(length_mm: number) {
    const pxPerMm = 8; const objects = [];
    objects.push(new fabric.Line([0, 0, length_mm * pxPerMm, 0], { stroke: '#000', strokeWidth: 1 }));
    for (let i = 0; i <= length_mm; i++) {
      const x = i * pxPerMm; let tickHeight = 5;
      if (i % 10 === 0) tickHeight = 12; else if (i % 5 === 0) tickHeight = 8;
      objects.push(new fabric.Line([x, 0, x, tickHeight], { stroke: '#000', strokeWidth: 1 }));
      if (i % 10 === 0) objects.push(new fabric.Text(i.toString(), { left: x, top: tickHeight + 2, fontSize: 10, fontFamily: 'sans-serif', originX: 'center' }));
    }
    const group = new fabric.Group(objects, { originX: 'center', originY: 'center' });
    group.set('data', { isRuler: true, length_mm });
    return group;
  }

  function addRuler(length_mm: number = 20) {
    if (!fabricCanvas) return;
    const ruler = createRulerGroup(length_mm);
    ruler.set({ left: WORKSPACE_W / 2, top: WORKSPACE_H / 2 });
    fabricCanvas.add(ruler); fabricCanvas.setActiveObject(ruler); fabricCanvas.renderAll(); debouncedUpdate();
  }

  async function searchIcons(query: string) {
    if (query.length < 2) { icon_results = []; return; }
    is_searching_icons = true;
    try {
      const response = await fetch(`https://api.iconify.design/search?query=${encodeURIComponent(query)}&limit=64`);
      const data = await response.json();
      if (data.icons) icon_results = data.icons.map((id: string) => { const [p, n] = id.split(':'); return { prefix: p, name: n, url: `https://api.iconify.design/${p}/${n}.svg` }; });
      else icon_results = [];
    } catch (err) { icon_results = []; } finally { is_searching_icons = false; }
  }

  $effect(() => {
    if (show_icons && icon_search.length >= 2) {
      const t = setTimeout(() => searchIcons(icon_search), 300);
      return () => clearTimeout(t);
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case 'f': e.preventDefault(); show_icons = true; setTimeout(() => document.getElementById('icon-search-input')?.focus(), 100); break;
        case 'b': if (active_object?.type === 'i-text') { e.preventDefault(); formatObject('fontWeight', ''); } break;
        case 'i': if (active_object?.type === 'i-text') { e.preventDefault(); formatObject('fontStyle', ''); } break;
        case 'p': e.preventDefault(); printLabel(); break;
      }
    } else {
      if ((e.key === 'Delete' || e.key === 'Backspace') && active_object && !(active_object as any).isEditing) {
        fabricCanvas?.remove(...fabricCanvas.getActiveObjects()); fabricCanvas?.discardActiveObject(); fabricCanvas?.renderAll();
      }
      if (active_object && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault(); const step = e.shiftKey ? 10 : 1;
        switch (e.key) {
          case 'ArrowUp': active_object.set('top', active_object.top! - step); break;
          case 'ArrowDown': active_object.set('top', active_object.top! + step); break;
          case 'ArrowLeft': active_object.set('left', active_object.left! - step); break;
          case 'ArrowRight': active_object.set('left', active_object.left! + step); break;
        }
        active_object.setCoords(); fabricCanvas?.renderAll(); debouncedUpdate();
      }
    }
  }

  function alignObject(alignment: string) {
    if (!fabricCanvas || !active_object) return;
    const l = WORKSPACE_W / 2 - label_width / 2;
    const t = WORKSPACE_H / 2 - label_height / 2;
    const rect = active_object.getBoundingRect();
    switch (alignment) {
      case 'left': active_object.set({ left: l + rect.width / 2 }); break;
      case 'center': active_object.set({ left: WORKSPACE_W / 2 }); break;
      case 'right': active_object.set({ left: l + label_width - rect.width / 2 }); break;
      case 'top': active_object.set({ top: t + rect.height / 2 }); break;
      case 'middle': active_object.set({ top: WORKSPACE_H / 2 }); break;
      case 'bottom': active_object.set({ top: t + label_height - rect.height / 2 }); break;
    }
    active_object.setCoords(); fabricCanvas.renderAll(); debouncedUpdate();
  }

  function formatObject(property: string, value: any) {
    if (!fabricCanvas || !active_object) return;
    if (property === 'fontWeight') value = (active_object as any).get(property) === 'bold' ? 'normal' : 'bold';
    else if (property === 'fontStyle') value = (active_object as any).get(property) === 'italic' ? 'normal' : 'italic';
    active_object.set(property as any, value); fabricCanvas.renderAll(); debouncedUpdate();
  }

  async function addIcon(url: string) {
    if (!fabricCanvas) return;
    try {
      const { objects, options } = await fabric.loadSVGFromURL(url);
      const icon = fabric.util.groupSVGElements(objects.filter(o => o !== null), options);
      icon.set({ left: WORKSPACE_W / 2, top: WORKSPACE_H / 2, originX: "center", originY: "center" });
      icon.scaleToWidth(label_height * 0.8);
      fabricCanvas.add(icon); fabricCanvas.setActiveObject(icon); fabricCanvas.renderAll(); show_icons = false;
    } catch (err) {}
  }

  function updatePixelData() {
    if (!fabricCanvas) return;
    const l = WORKSPACE_W / 2 - label_width / 2;
    const t = WORKSPACE_H / 2 - label_height / 2;
    const tempCanvas = document.createElement("canvas");
    tempCanvas.width = label_width; tempCanvas.height = label_height;
    const tempCtx = tempCanvas.getContext("2d"); if (!tempCtx) return;
    const fabricElement = fabricCanvas.toCanvasElement(1, { left: l, top: t, width: label_width, height: label_height });
    tempCtx.fillStyle = "#fff"; tempCtx.fillRect(0, 0, label_width, label_height);
    tempCtx.drawImage(fabricElement, 0, 0);
    pixelData = convertToData(tempCtx);
  }

  function debouncedUpdate() {
    if (updateTimeout) clearTimeout(updateTimeout);
    updateTimeout = setTimeout(() => updatePixelData(), 100) as any;
  }

  function addText(content: string = "New Text") {
    if (!fabricCanvas) return;
    const t = new fabric.IText(content, { left: WORKSPACE_W / 2, top: WORKSPACE_H / 2, fontFamily: "sans-serif", fontSize: font_size, fill: "#000", originX: "center", originY: "center", cornerStyle: "circle", transparentCorners: false, cornerColor: "#1565c0", cornerSize: 10 });
    fabricCanvas.add(t); fabricCanvas.setActiveObject(t); fabricCanvas.renderAll(); debouncedUpdate();
  }

  function addEmoji(emoji: string) {
    if (!fabricCanvas) return;
    const e = new fabric.Text(emoji, { left: WORKSPACE_W / 2, top: WORKSPACE_H / 2, fontSize: 64, originX: "center", originY: "center", cornerStyle: "circle", transparentCorners: false, cornerColor: "#1565c0", cornerSize: 10 });
    fabricCanvas.add(e); fabricCanvas.setActiveObject(e); fabricCanvas.renderAll(); debouncedUpdate();
  }

  function convertToData(ctx: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D) {
    const imageData = ctx.getImageData(0, 0, label_width, label_height);
    const dither = (x: number, y: number) => {
      const pattern = [[24, 406, 120, 502], [598, 215, 693, 311], [167, 550, 72, 454], [741, 359, 645, 263]];
      const p = label_width - x - 1 + y * label_width;
      const val = imageData.data[4 * p + 0] + imageData.data[4 * p + 1] + imageData.data[4 * p + 2];
      return val > pattern[x % 4][y % 4] ? 0 : 1;
    };
    const rows = Math.floor((label_height + 7) / 8);
    const pixels = new Uint8Array(rows * label_width);
    for (let x = 0, pos = 0; x < label_width; x++) {
      for (let y = 0; y < label_height; y += 8) {
        const val = dither(x, y) * 128 + dither(x, y + 1) * 64 + dither(x, y + 2) * 32 + dither(x, y + 3) * 16 + dither(x, y + 4) * 8 + dither(x, y + 5) * 4 + dither(x, y + 6) * 2 + dither(x, y + 7);
        pixels[pos] = val; pos++;
      }
    }
    return pixels;
  }

  onMount(() => {
    const barcode_font = new FontFace("Libre Barcode 39", "url('fonts/LibreBarcode39Text-Regular.ttf')");
    document.fonts.add(barcode_font);
    if (canvas) {
      fabricCanvas = new fabric.Canvas(canvas, { width: WORKSPACE_W, height: WORKSPACE_H, backgroundColor: "#eee", renderOnAddRemove: true, statefullCache: false, controlsAboveOverlay: true });
      const syncActive = () => { active_object = fabricCanvas?.getActiveObject(); };
      fabricCanvas.on("selection:created", syncActive); fabricCanvas.on("selection:updated", syncActive);
      fabricCanvas.on("selection:cleared", () => { active_object = undefined; debouncedUpdate(); });
      fabricCanvas.on("object:moving", (opt) => { if (opt.target) snapToGrid(opt.target); });
      fabricCanvas.on("object:scaling", (opt) => { if (opt.target) snapToGrid(opt.target); });
      fabricCanvas.on("object:modified", debouncedUpdate); fabricCanvas.on("object:added", debouncedUpdate); fabricCanvas.on("object:removed", debouncedUpdate);
      fabricCanvas.on("mouse:dblclick", (opt) => {
        const target = opt.target as any;
        if (target && target.data?.isRuler) {
          const newLenStr = prompt("New ruler length (mm):", target.data.length_mm.toString());
          if (newLenStr) {
            const newLen = parseInt(newLenStr);
            if (!isNaN(newLen) && newLen > 0) {
              const newRuler = createRulerGroup(newLen);
              newRuler.set({ left: target.left, top: target.top, angle: target.angle });
              fabricCanvas?.remove(target); fabricCanvas?.add(newRuler); fabricCanvas?.setActiveObject(newRuler); fabricCanvas?.renderAll(); debouncedUpdate();
            }
          }
        }
      });
      renderCanvasBackground(); addText("Label");
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  let selected_device: BluetoothDevice | undefined = $state();
  async function selectDevice() {
    if (is_tauri) {
      alert("Native App Mode: Your printer will be automatically detected when you click 'Print Label'.");
      return;
    }
    if (!navigator.bluetooth) { alert("Web Bluetooth not supported."); return; }
    try {
      const options = search_all_devices ? { acceptAllDevices: true, optionalServices: ["0000ff00-0000-1000-8000-00805f9b34fb"] } : { filters: [{ services: ["0000ff00-0000-1000-8000-00805f9b34fb"] }, { namePrefix: "Q30" }, { namePrefix: "D30" }, { namePrefix: "Phomemo" }], optionalServices: ["0000ff00-0000-1000-8000-00805f9b34fb"] };
      selected_device = await navigator.bluetooth.requestDevice(options);
      selected_device.addEventListener('gattserverdisconnected', () => { selected_device = undefined; });
    } catch (err) {}
  }
  function disconnectDevice() { if (selected_device?.gatt?.connected) selected_device.gatt.disconnect(); selected_device = undefined; }
  async function printLabel() {
    is_printing = true;
    try {
      updatePixelData();
      await tick();
      await new Promise(r => setTimeout(r, 150));

      console.log(`[Print] Preparing job. Pixels: ${pixelData.length} bytes.`);

      // Standard ESC/POS Protocol (ESC @ + GS v 0)
      const bytes_height = Math.floor((label_height + 7) / 8);
      const header = new Uint8Array([
        0x1b, 0x40, // Init
        0x1d, 0x76, 0x30, 0x00, // Print Raster
        bytes_height % 256, Math.floor(bytes_height / 256), 
        label_width % 256, Math.floor(label_width / 256)
      ]);
      const footer = new Uint8Array([0x1b, 0x64, 0x00]);

      if (is_tauri) {
        console.log("[Print] Native Bridge Mode");
        const fullBuffer = [...Array.from(header), ...Array.from(pixelData), ...Array.from(footer)];
        const result = await invoke("print_native", { data: fullBuffer });
        console.log(`[Print] Success: ${result}`);
        is_printing = false;
        return;
      }

      console.log("[Print] Web Bluetooth Mode");
      if (!navigator.bluetooth) { alert("Web Bluetooth not supported."); return; }
      
      if (!selected_device && navigator.bluetooth.getDevices) {
        const devices = await navigator.bluetooth.getDevices();
        selected_device = devices.find(d => d.name?.includes("Q30") || d.name?.includes("D30") || d.name?.includes("Phomemo"));
      }
      if (!selected_device) await selectDevice();
      if (!selected_device || !selected_device.gatt) throw new Error("No printer selected.");
      
      let server = selected_device.gatt;
      if (!server.connected) {
        console.log("[Print] Connecting to GATT...");
        server = await selected_device.gatt.connect();
      }
      
      console.log("[Print] Discovering Services...");
      const service = await server.getPrimaryService("0000ff00-0000-1000-8000-00805f9b34fb");
      const characteristic = await service.getCharacteristic("0000ff02-0000-1000-8000-00805f9b34fb");
      
      console.log("[Print] Streaming Header...");
      await characteristic.writeValueWithResponse(header);
      
      console.log("[Print] Streaming Pixels...");
      const chunkSize = 128;
      for (let i = 0; i < pixelData.length; i += chunkSize) {
        await characteristic.writeValueWithoutResponse(pixelData.slice(i, i + chunkSize));
      }
      
      console.log("[Print] Finalizing...");
      await characteristic.writeValueWithResponse(footer);
      console.log("[Print] Done!");
    } catch (err) {
      console.error("[Print] Error:", err);
      alert(`Printing failed: ${err}`);
      if (selected_device) disconnectDevice();
    } finally {
      is_printing = false;
    }
  }
</script>

<main>
  <h1>Web Label Printer</h1>
  <div class="r">
    <div class="toolbar-rows">
      <div class="bar creation-bar">
        <button onclick={() => addText()} class="add-text">Add Text</button>
        <button class="smile" onclick={() => { show_emoji = !show_emoji; show_icons = false; show_font_picker = false; }} aria-pressed={show_emoji} aria-label="Add Emoji"></button>
        <button class="icon-lib-btn" onclick={() => { show_icons = !show_icons; show_emoji = false; show_font_picker = false; }} aria-pressed={show_icons}>Icons</button>
        <div class="separator"></div>
        <div class="shape-tools">
          <button onclick={() => addShape('rect')} class="shape-btn" title="Rectangle">Rect</button>
          <button onclick={() => addShape('circle')} class="shape-btn" title="Circle">Circ</button>
          <button onclick={() => addShape('line')} class="shape-btn" title="Line">Line</button>
          <button onclick={() => { const len = prompt("Ruler length (mm):", "20"); if (len) addRuler(parseInt(len)); }} class="shape-btn" title="Add Ruler">Ruler</button>
        </div>
        <div class="separator"></div>
        <div class="grid-tools">
          <label class="grid-toggle"><input type="checkbox" bind:checked={grid_enabled} /> Grid</label>
          {#if grid_enabled}<input type="number" bind:value={grid_size} step="1" min="2" max="40" class="grid-size-input" />{/if}
        </div>
        <div class="separator"></div>
        <div class="border-tools">
          <select bind:value={border_style} class="border-select" title="Border Style">
            <option value="none">No Border</option><option value="solid">Solid</option><option value="dashed">Dashed</option><option value="dotted">Dotted</option><option value="double">Double</option><option value="brackets">Brackets</option><option value="thin">Thin</option><option value="rounded">Rounded</option>
          </select>
          {#if border_style !== 'none'}
            <label title="Thickness">T: <input type="number" bind:value={border_thickness} step="0.5" min="0.5" max="10" /></label>
            <label title="Margin (mm)">M: <input type="number" bind:value={border_margin} step="0.5" min="0" max="5" /></label>
          {/if}
        </div>
      </div>

      <div class="bar action-bar" class:hidden={!active_object}>
        {#if active_object}
          <div class="context-tools">
            {#if active_object.type === 'i-text'}
              <select value={(active_object as any).fontFamily} onchange={(e) => handleFontChange(e.currentTarget.value)} class="font-select">
                {#each FONTS as font}<option value={font}>{font}</option>{/each}
              </select>
              <button onclick={() => formatObject('fontWeight', '')} class="bold" title="Bold"></button>
              <button onclick={() => formatObject('fontStyle', '')} class="italic" title="Italic"></button>
            {/if}
            <div class="align-group">
              <button onclick={() => alignObject('left')} class="align-left" title="Align Left"></button>
              <button onclick={() => alignObject('center')} class="align-center" title="Align Center"></button>
              <button onclick={() => alignObject('right')} class="align-right" title="Align Right"></button>
              <button onclick={() => alignObject('middle')} class="align-middle" title="Center Vertical"></button>
            </div>
            <div class="separator"></div>
            <div class="layer-tools">
              <button onclick={() => { if (active_object) { fabricCanvas?.bringObjectToFront(active_object); fabricCanvas?.renderAll(); } }} title="Bring to Front">⇈</button>
              <button onclick={() => { if (active_object) { fabricCanvas?.sendObjectToBack(active_object); renderCanvasBackground(); addBorder(border_style); } }} title="Send to Back">⇊</button>
            </div>
            <button onclick={() => { if (fabricCanvas) { const activeObjects = fabricCanvas.getActiveObjects(); fabricCanvas.remove(...activeObjects); fabricCanvas.discardActiveObject(); fabricCanvas.renderAll(); } }} class="delete-obj" title="Delete Selected">Delete</button>
          </div>
        {/if}
      </div>
    </div>

    {#if show_emoji}<Emoji onselect={(emoji: string) => { addEmoji(emoji); show_emoji = false; }} />{/if}
    {#if show_font_picker}
      <div class="font-picker">
        <div class="picker-header">
          <input 
            type="text" 
            bind:value={font_search} 
            placeholder="Search thousands of fonts..." 
            onfocus={() => fetchGoogleFonts()}
            oninput={() => font_page = 0}
          />
          <button onclick={() => show_font_picker = false}>Close</button>
        </div>
        {#if is_loading_fonts}
          <div class="picker-status">Loading font library...</div>
        {:else}
          <div class="font-grid">
            {#each filtered_fonts as family}
              <button 
                onclick={() => loadGoogleFont(family)}
                class="font-item"
                class:active={(active_object as any)?.fontFamily === family}
                style="font-family: '{family}', sans-serif;"
              >
                {family}
              </button>
            {/each}
          </div>
          {#if total_font_pages > 1}
            <div class="picker-footer">
              <button onclick={() => font_page = Math.max(0, font_page - 1)} disabled={font_page === 0}>Prev</button>
              <span>Page {font_page + 1} of {total_font_pages}</span>
              <button onclick={() => font_page = Math.min(total_font_pages - 1, font_page + 1)} disabled={font_page >= total_font_pages - 1}>Next</button>
            </div>
          {/if}
        {/if}
      </div>
    {/if}
    {#if show_icons}
      <div class="icon-library">
        <div class="icon-search-bar"><input id="icon-search-input" type="text" bind:value={icon_search} placeholder="Search 200,000+ icons (Ctrl+F)..." autocomplete="off" /><button onclick={() => { icon_search = ""; show_icons = false; }}>Close</button></div>
        {#if is_searching_icons}<div class="icon-loading">Searching...</div>
        {:else if icon_results.length > 0}
          <div class="icon-grid">{#each icon_results as icon}<button onclick={() => addIcon(icon.url)} class="icon-item" title={`${icon.prefix}:${icon.name}`}><img src={icon.url} alt={icon.name} width="24" height="24" /><span>{icon.name}</span></button>{/each}</div>
        {:else if icon_search.length >= 2}<div class="icon-empty">No icons found for "{icon_search}"</div>
        {:else}<div class="icon-empty">Type at least 2 characters to search...</div>{/if}
      </div>
    {/if}

    <div class="canvas-wrapper"><canvas bind:this={canvas}></canvas></div>

    <div class="template-section">
      <button onclick={saveTemplate} class="save-template-btn">Save as Template</button>
      {#if templates.length > 0}
        <div class="template-list">
          {#each templates as template, i}
            <div class="template-item"><span class="template-name">{template.name}</span><div class="template-actions"><button onclick={() => loadTemplate(template.data)} class="load-btn">Load</button><button onclick={() => deleteTemplate(i)} class="delete-btn">×</button></div></div>
          {/each}
        </div>
      {/if}
    </div>

    <details>
      <summary>⚙️ Configuration</summary>
      <div class="config">
        <label>Label Width (mm): <input type="number" bind:value={label_width_mm} min="1" max="150" step="0.1" /></label>
        <label>Label Height (mm): <input type="number" bind:value={label_height_mm} min="1" max="18" step="0.1" /></label>
        <label><input type="checkbox" bind:checked={search_all_devices} /> Show All Bluetooth Devices (Troubleshooting)</label>
        <p style="font-size: 0.8em; color: #666; margin: 0;">Tip: Ensure the printer is <strong>unpaired/disconnected</strong> from your system's Bluetooth settings before searching.</p>
        <button 
          onclick={async () => {
            alert("Starting hardware diagnostic... Check terminal for logs.");
            try {
              const res = await invoke("test_autopilot");
              alert(res);
            } catch (e) {
              alert("Diagnostic failed: " + e);
            }
          }}
          style="margin-top: 10px; background: #fff3e0; color: #e65100; border-color: #ffe0b2;"
        >
          Run Hardware Diagnostic
        </button>
      </div>
    </details>

    <div class="actions">
      {#if selected_device}<button onclick={disconnectDevice} class="disconnect-btn">Disconnect {selected_device.name || "Printer"}</button>
      {:else}<button onclick={selectDevice} class="connect-btn">Connect Printer</button>{/if}
      <button onclick={printLabel} class="print-btn" disabled={is_printing}>{is_printing ? "Printing..." : "Print Label"}</button>
    </div>

    <div class="l"><span>Label size: {(label_width * 0.125).toFixed(1)} × {(label_height * 0.125).toFixed(1)} mm</span></div>
  </div>
</main>

<style>
  * { box-sizing: border-box; }
  h1 { font-size: 3.2em; line-height: 1.1; border-bottom: 1px solid #444; }
  .toolbar-rows { display: flex; flex-direction: column; gap: 8px; width: 100%; flex-shrink: 0; }
  div.bar { display: flex; flex-direction: row; flex-wrap: wrap; align-items: center; justify-content: start; column-gap: 8px; row-gap: 4px; width: 100%; min-height: 50px; padding: 4px 10px; background: #f5f5f5; border-radius: 12px; border: 1px solid #ccc; flex-shrink: 0; }
  .action-bar.hidden { visibility: hidden; }
  .context-tools { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; width: 100%; }
  .align-group { display: flex; border: 1px solid #ccc; border-radius: 8px; overflow: hidden; flex-shrink: 0; }
  .align-group button { border: none; border-radius: 0; border-right: 1px solid #ccc; padding: 4px 8px; }
  .align-group button:last-child { border-right: none; }
  .shape-tools { display: flex; gap: 4px; flex-shrink: 0; }
  .shape-btn { font-size: 0.75em; padding: 4px 6px; background-color: #eee; }
  .font-select, .border-select { padding: 4px; border-radius: 6px; border: 1px solid #ccc; background: #fff; font-family: inherit; font-size: 0.85em; flex-shrink: 0; }
  .template-section { width: 100%; margin-top: 10px; padding: 10px; background: #f9f9f9; border: 1px solid #eee; border-radius: 12px; }
  .template-list { display: flex; flex-direction: column; gap: 4px; margin-top: 10px; }
  .template-item { display: flex; justify-content: space-between; align-items: center; padding: 4px 8px; background: #fff; border: 1px solid #eee; border-radius: 8px; font-size: 0.9em; }
  .template-actions { display: flex; gap: 4px; }
  .save-template-btn { width: 100%; background-color: #e8f5e9; color: #2e7d32; border-color: #c8e6c9; padding: 6px; }
  .load-btn { padding: 2px 6px; font-size: 0.8em; background-color: #e3f2fd; color: #1565c0; }
  .template-item .delete-btn { padding: 2px 6px; font-size: 0.8em; background-color: #ffebee; color: #c62828; border: none; }
  .font-picker { width: 100%; padding: 10px; background: #fff; border: 1px solid #ddd; border-radius: 12px; max-height: 300px; display: flex; flex-direction: column; gap: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.15); z-index: 100; }
  .picker-header { display: flex; gap: 6px; position: sticky; top: 0; background: white; padding-bottom: 6px; border-bottom: 1px solid #eee; }
  .picker-header input { flex: 1; padding: 4px 8px; border: 1px solid #ccc; border-radius: 6px; font-size: 0.9em; }
  .font-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 6px; overflow-y: auto; flex: 1; }
  .picker-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 8px;
    border-top: 1px solid #eee;
    font-size: 0.85em;
    color: #666;
  }
  .picker-footer button {
    padding: 2px 8px;
    font-size: 0.9em;
  }
  .font-item { padding: 6px !important; text-align: left; background: #f9f9f9; border: 1px solid #eee; font-size: 13px; justify-content: flex-start !important; }
  .font-item:hover { background: #eee; }
  .font-item.active {
    background-color: #e3f2fd !important;
    border-color: #1565c0 !important;
    font-weight: bold;
  }
  .picker-status { padding: 10px; text-align: center; font-style: italic; color: #666; font-size: 0.9em; }
  .separator { width: 1px; height: 24px; background-color: #ccc; margin: 0 2px; flex-shrink: 0; }
  .icon-library { width: 100%; padding: 10px; background: #fff; border: 1px solid #ddd; border-radius: 12px; max-height: 300px; display: flex; flex-direction: column; gap: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.15); z-index: 100; }
  .icon-search-bar { display: flex; gap: 6px; position: sticky; top: 0; background: white; padding-bottom: 6px; border-bottom: 1px solid #eee; }
  .icon-search-bar input { flex: 1; padding: 4px 8px; border: 1px solid #ccc; border-radius: 6px; font-size: 0.9em; }
  .icon-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(70px, 1fr)); gap: 6px; overflow-y: auto; padding-right: 4px; }
  .icon-item { display: flex; flex-direction: column; align-items: center; gap: 2px; font-size: 0.65em; padding: 4px 2px !important; background: #f9f9f9; border: 1px solid #eee; height: auto !important; }
  .icon-item img { filter: grayscale(1); }
  .icon-item:hover { background: #eee; }
  .icon-item span { width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; text-align: center; }
  .icon-loading, .icon-empty { padding: 10px; text-align: center; color: #666; font-style: italic; font-size: 0.9em; }
  .border-tools, .grid-tools { display: flex; align-items: center; gap: 4px; font-size: 0.75em; flex-shrink: 0; }
  .grid-toggle { display: flex; align-items: center; gap: 2px; cursor: pointer; }
  .grid-size-input, .border-tools input { width: 34px; padding: 2px; border: 1px solid #ccc; border-radius: 4px; }
  .layer-tools { display: flex; gap: 2px; flex-shrink: 0; }
  .layer-tools button { padding: 2px 6px; font-size: 1.1em; background: #eee; }
  div.r { display: flex; flex-direction: column; align-items: center; justify-content: start; gap: 1em; width: 800px; margin: 0 auto; position: relative; }
  .canvas-wrapper { width: 800px; height: 200px; border: 1px solid #ccc; background-color: #eee; display: flex; align-items: center; justify-content: center; overflow: visible; position: relative; box-shadow: inset 0 0 15px rgba(0,0,0,0.1); }
  .canvas-wrapper canvas { box-shadow: 0 0 10px rgba(0,0,0,0.2); }
  .l { width: 100%; text-align: center; margin-top: 4px; }
  .l span { color: #888; font-size: 0.85em; }
  button { padding: 0.4em 0.8em; font-size: 0.9em; font-weight: 500; font-family: inherit; background-color: #ddd; border: 1px solid #ccc; border-radius: 8px; cursor: pointer; transition: all 0.2s ease; display: flex; align-items: center; justify-content: center; }
  button:hover { background-color: #ccc; }
  button[aria-pressed="true"] { background-color: #def; border-color: #1565c0; }
  .actions { display: flex; gap: 0.8em; width: 100%; margin-top: 10px; }
  .actions button { flex: 1; padding: 0.8em; }
  .connect-btn { background-color: #f0f0f0; color: #333; }
  .disconnect-btn { background-color: #fff3e0; color: #e65100; border-color: #ffe0b2; }
  .print-btn { background-color: #4caf50; color: white; border-color: #45a049; font-weight: bold; }
  .print-btn:disabled { background-color: #a5d6a7; cursor: not-allowed; }
  .delete-obj { background-color: #ffcdd2; color: #c62828; border-color: #ef9a9a; margin-left: auto; }
  .add-text { background-color: #e3f2fd; color: #1565c0; border-color: #bbdefb; }
  .bold, .italic, .align-left, .align-center, .align-right, .align-middle, .smile { width: 32px; height: 32px; padding: 0 !important; background-repeat: no-repeat !important; background-position: center !important; background-size: 18px !important; }
  .italic { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxsaW5lIHgxPSIxOSIgeDI9IjEwIiB5MT0iNCIgeTI9IjQiLz48bGluZSB4MT0iMTQiIHgyPSI1IiB5MT0iMjAiIHkyPSIyMCIvPjxsaW5lIHgxPSIxNSIgeDI9IjkiIHkxPSI0IiB5Mj0iMjAiLz48L3N2Zz4="); }
  .bold { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxwYXRoIGQ9Ik02IDEyOWE0IDQgMCAwIDEgMCA4SDdhMSAxIDAgMCAxLTEtMVY1YTEgMSAwIDAgMSAxLTFoN2E0IDQgMCAwIDEgMCA4Ii8+PC9zdmc+"); }
  .smile { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxjaXJjbGUgY3g9IjEyIiBjeT0iMTIiIHI9IjEwIi8+PHBhdGggZD0iTTggMTRzMS41IDIgNCAyIDQtMiA0LTIiLz48bGluZSB4MT0iOSIgeDI9IjkuMDEiIHkxPSI5IiB5Mj0iOSIvPjxsaW5lIHgxPSIxNSIgeDI9IjE1LjAxIiB5MT0iOSIgeTI9IjkiLz48L3N2Zz4="); }
  .align-left { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxsaW5lIHgxPSIxNyIgeDI9IjMiIHkxPSI2IiB5Mj0iNiIvPjxsaW5lIHgxPSIxOSIgeDI9IjMiIHkxPSIxMiIgeTI9IjEyIi8+PGxpbmUgeDE9IjExIiB4Mj0iMyIgeTE9IjE4IiB5Mj0iMTgiLz48L3N2Zz4="); }
  .align-center { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxsaW5lIHgxPSIxNyIgeDI9IjMiIHkxPSI2IiB5Mj0iNiIvPjxsaW5lIHgxPSIxOSIgeDI9IjMiIHkxPSIxMiIgeTI9IjEyIi8+PGxpbmUgeDE9IjExIiB4Mj0iMyIgeTE9IjE4IiB5Mj0iMTgiLz48L3N2Zz4="); }
  .align-right { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxsaW5lIHgxPSIxNyIgeDI9IjMiIHkxPSI2IiB5Mj0iNiIvPjxsaW5lIHgxPSIxOSIgeDI9IjMiIHkxPSIxMiIgeTI9IjEyIi8+PGxpbmUgeDE9IjExIiB4Mj0iMyIgeTE9IjE4IiB5Mj0iMTgiLz48L3N2Zz4="); }
  .align-middle { background-image: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxyZWN0IHdpZHRoPSIyMCIgaGVpZ2h0PSIxNiIgeD0iMiIgeT0iMyIgcng9IjIiLz48bGluZSB4MT0iMiIgeDI9IjIyIiB5MT0iMTEiIHkyPSIxMSIvPjwvc3ZnPg=="); }
  details { width: 100%; box-sizing: border-box; margin-top: 10px; }
  .config { display: flex; flex-direction: column; gap: 0.5em; padding: 1em; border: 1px solid #ddd; border-radius: 12px; background-color: #f9f9f9; }
  .config label { display: flex; justify-content: space-between; align-items: center; font-weight: 500; font-size: 0.9em; }
  .config input { width: 70px; padding: 0.25em; border: 1px solid #ccc; border-radius: 6px; text-align: center; }
</style>
