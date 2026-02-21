<script lang="ts">
  import { onMount, settled, tick } from "svelte";
  import * as fabric from "fabric";
  import Emoji from "./lib/Emoji.svelte";

  type CanvasContext =
    | CanvasRenderingContext2D
    | OffscreenCanvasRenderingContext2D;

  // Dimensions in mm (loaded from localStorage)
  let label_width_mm = $state(
    parseFloat(localStorage.getItem("label_width_mm") || "40"),
  );
  let label_height_mm = $state(
    parseFloat(localStorage.getItem("label_height_mm") || "12"),
  );
  let margin_height_mm = $state(
    parseFloat(localStorage.getItem("margin_height_mm") || "1"),
  );
  let margin_width_mm = $state(
    parseFloat(localStorage.getItem("margin_mm") || "1"),
  );

  // Width and height in 0.125 mm units (derived, rounded to integers)
  let label_width = $derived(Math.round(label_width_mm * 8));
  let label_height = $derived(Math.round(label_height_mm * 8));
  let margin_x = $derived(Math.round(margin_width_mm * 8));
  let margin_y = $derived(Math.round(margin_height_mm * 8));
  let text = $state(sessionStorage.getItem("text") || "");
  let show_emoji = $state(false);
  let textarea: HTMLTextAreaElement | undefined = $state();

  // Font size settings (loaded from localStorage)
  let auto_font_size = $state(
    localStorage.getItem("auto_font_size") !== "false",
  );
  let font_size = $state(parseFloat(localStorage.getItem("font_size") || "48"));
  let search_all_devices = $state(
    localStorage.getItem("search_all_devices") === "true",
  );
  let horizontal_align = $state(localStorage.getItem("horizontal_align") || "center");
  let vertical_align = $state(localStorage.getItem("vertical_align") || "middle");
  let selected_device: BluetoothDevice | undefined = $state();

  // Save to localStorage
  $effect(() => {
    localStorage.setItem("label_width_mm", label_width_mm.toString());
    localStorage.setItem("label_height_mm", label_height_mm.toString());
    localStorage.setItem("margin_mm", margin_width_mm.toString());
    localStorage.setItem("margin_height_mm", margin_height_mm.toString());
  });

  // Save font settings to localStorage
  $effect(() => {
    localStorage.setItem("auto_font_size", auto_font_size.toString());
    localStorage.setItem("font_size", font_size.toString());
    localStorage.setItem("search_all_devices", search_all_devices.toString());
    localStorage.setItem("horizontal_align", horizontal_align);
    localStorage.setItem("vertical_align", vertical_align);
  });

  // Save text to sessionStorage
  $effect(() => {
    sessionStorage.setItem("text", text);
  });
  let canvas: HTMLCanvasElement | undefined = $state();
  let pixelData = new Uint8Array();

  let editor_mode = $state(localStorage.getItem("editor_mode") || "visual");
  let visual_editor: HTMLDivElement | undefined = $state();

  $effect(() => {
    localStorage.setItem("editor_mode", editor_mode);
  });

  function toHtml(md: string) {
    return md
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/\[\|(.*?)\|\]/g, '<span class="html-barcode">$1</span>')
      .replace(/\^\^(.*?)\^\^/g, '<span class="html-big">$1</span>')
      .replace(/__(.*?)__/g, '<span class="html-small">$1</span>')
      .replace(/\*(.*?)\*/g, "<strong>$1</strong>")
      .replace(/_(.*?)_/g, "<em>$1</em>")
      .replace(/\n/g, "<br>");
  }

  function fromHtml(html: string) {
    return html
      .replace(/<br\s*\/?>/gi, "\n")
      .replace(/<div>(.*?)<\/div>/gi, "\n$1")
      .replace(/<span class="html-barcode">(.*?)<\/span>/gi, "[|$1|]")
      .replace(/<span class="html-big">(.*?)<\/span>/gi, "^^$1^^")
      .replace(/<span class="html-small">(.*?)<\/span>/gi, "__$1__")
      .replace(/<strong>(.*?)<\/strong>/gi, "*$1*")
      .replace(/<b>(.*?)<\/b>/gi, "*$1*")
      .replace(/<em>(.*?)<\/em>/gi, "_$1_")
      .replace(/<i>(.*?)<\/i>/gi, "_$1_")
      .replace(/&nbsp;/g, " ")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&amp;/g, "&")
      .replace(/<[^>]*>/g, ""); // Strip remaining tags
  }

  function renderLine(
    ctx: CanvasContext,
    line: string,
    size: number,
    render_fn: (str: string) => void,
  ) {
    // Split line into components:
    const match_bold = /(?<!\*)\*(?! )([^*]*?[^* ]\*)/g;
    const match_italic = /(?<!_)_(?! )([^_]*?[^_ ]_)/g;
    const match_barcode = /\[\|.*?\|\]/g;
    const match_big = /\^\^.*?\^\^/g;
    const match_small = /__.*?__/g;

    const parsed = [];

    // Start by matching "barcode" tags to avoid matching inside
    var clean_line = line;
    for (const m of clean_line.matchAll(match_barcode)) {
      // Add to parsed array
      const start = m.index;
      const end = m.index + m[0].length;
      parsed.push({ pos: start, type: "barcode" });
      parsed.push({ pos: end - 2, type: "barcode end" });
      clean_line =
        clean_line.substring(0, start) +
        "\n".repeat(m[0].length) +
        clean_line.substring(end);
    }

    // Now, add all matches
    const add = (type: string, regex: RegExp, ln: number) => {
      for (const m of clean_line.matchAll(regex)) {
        const start = m.index;
        const end = m.index + m[0].length;
        parsed.push({ pos: start, type });
        parsed.push({ pos: end - ln, type: type + " end" });
      }
    };

    add("big", match_big, 2);
    add("small", match_small, 2);
    add("bold", match_bold, 1);
    add("italic", match_italic, 1);

    // Add a last tag at string end
    parsed.push({ pos: line.length, type: "" });

    // Sort tags
    const tags = parsed.sort((a, b) => a.pos - b.pos);

    // Now, process the line keeping the state of the font
    var pos = 0;
    var current_font = { size, name: "sans", italic: false, bold: false };
    const font_stack: (typeof current_font)[] = [];
    const make_font = (f: typeof current_font) => {
      if (f.name.search(" "))
        return `${f.italic ? "italic" : ""} ${f.bold ? "bold" : ""} ${f.size}px "${f.name}"`;
      else
        return `${f.italic ? "italic" : ""} ${f.bold ? "bold" : ""} ${f.size}px ${f.name}`;
    };
    for (const tag of tags) {
      if (tag.pos > pos) {
        // Add current string
        ctx.font = make_font(current_font);
        render_fn(line.substring(pos, tag.pos));
      }
      // Modify font
      switch (tag.type) {
        case "bold":
          current_font.bold = true;
          pos = tag.pos + 1;
          break;
        case "italic":
          current_font.italic = true;
          pos = tag.pos + 1;
          break;
        case "big":
          current_font.size *= 3.0 / 2.0;
          pos = tag.pos + 2;
          break;
        case "small":
          current_font.size *= 2.0 / 3.0;
          pos = tag.pos + 2;
          break;
        case "bold end":
          current_font.bold = false;
          pos = tag.pos + 1;
          break;
        case "italic end":
          current_font.italic = false;
          pos = tag.pos + 1;
          break;
        case "big end":
          current_font.size *= 2.0 / 3.0;
          pos = tag.pos + 2;
          break;
        case "small end":
          current_font.size *= 3.0 / 2.0;
          pos = tag.pos + 2;
          break;
        case "barcode":
          font_stack.push(current_font);
          current_font.bold = current_font.italic = false;
          current_font.name = "Libre Barcode 39";
          pos = tag.pos + 2;
          break;
        case "barcode end":
          const s = font_stack.pop();
          if (s) current_font = s;
          pos = tag.pos + 2;
          break;
      }
    }
  }

  function measureLine(ctx: CanvasContext, line: string, size: number) {
    // To properly measure, we need to return:
    //  - Right as the first actualBoundingBoxRight;
    //  - Left as the sum of all the widths minus the last width and plus the last actualBoundingBoxLeft.

    var first = true;
    var right = 0;
    var left = 0;
    var ascent = 0;
    var descent = 0;
    var m: TextMetrics | undefined;
    renderLine(ctx, line, size, (str) => {
      m = ctx.measureText(str);
      if (first) {
        ascent = m.actualBoundingBoxAscent;
        descent = m.actualBoundingBoxDescent;
        left = m.actualBoundingBoxLeft;
        right = m.width;
        first = false;
      } else {
        ascent = Math.max(ascent, m.actualBoundingBoxAscent);
        descent = Math.max(descent, m.actualBoundingBoxDescent);
        right += m.width;
      }
    });
    if (m !== undefined) {
      right += m.actualBoundingBoxRight - m.width;
    }
    return { right, left, ascent, descent };
  }

  function fullSize(ctx: CanvasContext, lines: string[], sz: number) {
    const line_hg = Math.round((sz * 7) / 6);
    var wd = 0;
    var total_hg = 0;
    // Get all lines
    for (var i = 0; i < lines.length; i++) {
      const m = measureLine(ctx, lines[i], sz);
      wd = Math.max(wd, m.right + m.left);
      total_hg += i == 0 ? m.ascent : line_hg;
      if (i == lines.length - 1) {
        total_hg += m.descent;
      }
    }
    return { wd, line_hg, total_hg };
  }

  function bestSize(
    ctx: CanvasContext,
    lines: string[],
    min_size = 8,
    max_size = 144,
  ) {
    var wd = ctx.canvas.width - margin_x * 2;
    var hg = ctx.canvas.height - margin_y * 2;

    while (min_size + 0.1 < max_size) {
      var size = (min_size + max_size) / 2;
      var s = fullSize(ctx, lines, size);
      if (s.wd >= wd || s.total_hg >= hg) {
        max_size = size;
      } else {
        min_size = size;
      }
    }
    return min_size;
  }

  function drawText(ctx: CanvasContext, text: string) {
    ctx.fillStyle = "#fff";
    ctx.fillRect(0, 0, label_width, label_height);

    // Draw text lines:
    const lines = text.split("\n");
    const sz = auto_font_size ? bestSize(ctx, lines) : font_size;
    const font_sz = fullSize(ctx, lines, sz);

    ctx.textBaseline = "alphabetic";
    ctx.fillStyle = "#000";

    // Vertical alignment calculation
    var y = margin_y;
    if (vertical_align === "middle") {
      y = (label_height - font_sz.total_hg) / 2;
    } else if (vertical_align === "bottom") {
      y = label_height - font_sz.total_hg - margin_y;
    }

    for (var i = 0; i < lines.length; i++) {
      const line = lines[i];

      // We have to render twice, once to measure, another to draw
      const m = measureLine(ctx, line, sz);

      const line_wd = m.right + m.left;

      // Horizontal alignment calculation
      var x = margin_x + m.left;
      if (horizontal_align === "center") {
        x = (label_width - line_wd) / 2 + m.left;
      } else if (horizontal_align === "right") {
        x = label_width - line_wd - margin_x + m.left;
      }

      y += i ? font_sz.line_hg : m.ascent;

      renderLine(ctx, lines[i], sz, (str) => {
        const m = ctx.measureText(str);
        ctx.fillText(str, x, y);
        x += m.width;
      });
    }
  }

  function convertToData(ctx: CanvasContext) {
    // Get image data
    const imageData = ctx.getImageData(0, 0, label_width, label_height);

    // Dither pattern
    const dither = (x: number, y: number) => {
      const pattern = [
        [24, 406, 120, 502],
        [598, 215, 693, 311],
        [167, 550, 72, 454],
        [741, 359, 645, 263],
      ];
      const p = label_width - x - 1 + y * label_width;
      const val =
        imageData.data[4 * p + 0] + // red
        imageData.data[4 * p + 1] + // green
        imageData.data[4 * p + 2]; // blue
      return val > pattern[x % 4][y % 4] ? 0 : 1;
    };

    // Dither to binary image data, in the printer format
    const rows = Math.floor((label_height + 7) / 8);
    const pixels = new Uint8Array(rows * label_width);
    for (let x = 0, pos = 0; x < label_width; x++) {
      for (let y = 0; y < label_height; y += 8) {
        const val =
          dither(x, y) * 128 +
          dither(x, y + 1) * 64 +
          dither(x, y + 2) * 32 +
          dither(x, y + 3) * 16 +
          dither(x, y + 4) * 8 +
          dither(x, y + 5) * 4 +
          dither(x, y + 6) * 2 +
          dither(x, y + 7);
        pixels[pos] = val;
        pos++;
      }
    }

    return pixels;
  }

  function draw() {
    if (!canvas) return;

    const offscreenCanvas = new OffscreenCanvas(label_width, label_height);
    const off_ctx = offscreenCanvas.getContext("2d");
    if (!off_ctx) return;

    drawText(off_ctx, text);
    pixelData = convertToData(off_ctx);

    // Convert back to image data for preview
    const img = new ImageData(label_width, label_height);

    const setPix = (x: number, y: number, v: number) => {
      const p = v ? 0 : 255;
      const t = label_width - 1 - x + y * label_width;
      img.data[t * 4 + 0] = p;
      img.data[t * 4 + 1] = p;
      img.data[t * 4 + 2] = p;
      img.data[t * 4 + 3] = 255;
    };

    for (let x = 0, pos = 0; x < label_width; x++) {
      for (let y = 0; y < label_height; y += 8) {
        const val = pixelData[pos];
        setPix(x, y + 0, val & 128);
        setPix(x, y + 1, val & 64);
        setPix(x, y + 2, val & 32);
        setPix(x, y + 3, val & 16);
        setPix(x, y + 4, val & 8);
        setPix(x, y + 5, val & 4);
        setPix(x, y + 6, val & 2);
        setPix(x, y + 7, val & 1);
        pos++;
      }
    }

    // Copy to preview canvas
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.putImageData(img, 0, 0);
  }

  $effect(() => {
    draw();
  });

  onMount(() => {
    // Force loading of font at start, so we don't need to include in the HTML
    const barcode_font = new FontFace(
      "Libre Barcode 39",
      "url('fonts/LibreBarcode39Text-Regular.ttf')",
    );
    document.fonts.add(barcode_font);
    barcode_font.load().then(() => {
      draw();
    });
  });

  async function selectDevice() {
    if (!navigator.bluetooth) {
      alert(
        "Web Bluetooth is not supported by your browser or in this context (requires HTTPS or localhost).",
      );
      return;
    }
    const options = search_all_devices
      ? {
          acceptAllDevices: true,
          optionalServices: ["0000ff00-0000-1000-8000-00805f9b34fb"],
        }
      : {
          filters: [
            { services: ["0000ff00-0000-1000-8000-00805f9b34fb"] },
            { namePrefix: "Q30" },
            { namePrefix: "D30" },
            { namePrefix: "Phomemo" },
          ],
          optionalServices: ["0000ff00-0000-1000-8000-00805f9b34fb"],
        };
    selected_device = await navigator.bluetooth.requestDevice(options);
  }

  async function printLabel() {
    if (!navigator.bluetooth) {
      alert(
        "Web Bluetooth is not supported by your browser or in this context (requires HTTPS or localhost).",
      );
      return;
    }
    try {
      // Try to find a previously paired device if none is selected
      if (!selected_device && navigator.bluetooth.getDevices) {
        const devices = await navigator.bluetooth.getDevices();
        selected_device = devices.find(
          (d) =>
            d.name?.includes("Q30") ||
            d.name?.includes("D30") ||
            d.name?.includes("Phomemo"),
        );
      }

      // If still no device, ask user to select one
      if (!selected_device) {
        await selectDevice();
      }

      if (!selected_device || !selected_device.gatt) {
        throw new Error("No Bluetooth device selected or supported.");
      }
      const server = await selected_device.gatt.connect();
      const service = await server.getPrimaryService(
        "0000ff00-0000-1000-8000-00805f9b34fb",
      );
      const characteristic = await service.getCharacteristic(
        "0000ff02-0000-1000-8000-00805f9b34fb",
      );

      // Ok, will print our data
      const bytes_height = Math.floor((label_height + 7) / 8);
      // ESC/POS reference:
      //   https://download4.epson.biz/sec_pubs/pos/reference_en/escpos/commands.html
      const header = new Uint8Array([
        0x1b, // ESC @ : initialize printer
        0x40,
        0x1d, // GS v 0 : Print raster image
        0x76,
        0x30,
        0x00, // normal image
        bytes_height % 256, // bytes in horizontal direction
        Math.floor(bytes_height / 256),
        label_width % 256, // lines in vertical direction
        Math.floor(label_width / 256),
      ]);
      // ESC d 00 : Print and feed 0 lines.
      const footer = new Uint8Array([0x1b, 0x64, 0x00]);

      await characteristic.writeValueWithResponse(header);
      for (let i = 0; i < pixelData.length; i += 128) {
        const buf = pixelData.slice(i, i + 128);
        // TODO: last packet should be padded??
        await characteristic.writeValueWithoutResponse(buf);
      }
      await characteristic.writeValueWithResponse(footer);
    } catch (err) {
      alert(`Error printing: ${err}`);
      console.error(`Error printing: ${err}`);
    }
  }

  async function insertText(str: string) {
    if (!textarea) return;
    const pos0 = textarea.selectionStart ?? 0;
    const pos1 = textarea.selectionEnd ?? 0;
    text = text.substring(0, pos1) + str + text.substring(pos1);
    await settled();
    textarea.setSelectionRange(pos0, pos1 + str.length);
    textarea.focus();
  }

  async function setAttribute(attr1: string, attr2 = attr1) {
    if (!textarea) return;
    let pos0 = textarea.selectionStart ?? 0;
    let pos1 = textarea.selectionEnd ?? 0;
    const sel = text.substring(pos0, pos1);

    // Check if selection already has attribute
    if (sel.startsWith(attr1) && sel.endsWith(attr2)) {
      text =
        text.substring(0, pos0) +
        sel.substring(attr1.length, sel.length - attr2.length) +
        text.substring(pos1);
      pos1 = pos1 - attr1.length - attr2.length;
    } else {
      text =
        text.substring(0, pos0) + attr1 + sel + attr2 + text.substring(pos1);
      pos1 = pos1 + attr1.length + attr2.length;
    }
    await settled();
    textarea.setSelectionRange(pos0, pos1);
    textarea.focus();
  }
</script>

<main>
  <h1>Web Label Printer</h1>
  <div class="r">
    <div class="editor-container">
      <div class="editor-tabs">
        <button
          class="tab-btn"
          aria-pressed={editor_mode === "visual"}
          onclick={() => {
            if (editor_mode === "raw" && visual_editor) {
              visual_editor.innerHTML = toHtml(text);
            }
            editor_mode = "visual";
          }}
        >
          Visual Editor
        </button>
        <button
          class="tab-btn"
          aria-pressed={editor_mode === "raw"}
          onclick={() => {
            if (editor_mode === "visual" && visual_editor) {
              text = fromHtml(visual_editor.innerHTML);
            }
            editor_mode = "raw";
          }}
        >
          Raw Text (Markdown)
        </button>
      </div>

      <div class="bar">
        {#if editor_mode === "visual"}
          <button
            onclick={() => document.execCommand("bold")}
            class="bold"
            aria-label="Bold Text"
          ></button>
          <button
            onclick={() => document.execCommand("italic")}
            class="italic"
            aria-label="Italic Text"
          ></button>
          <button
            onclick={() => {
              const sel = window.getSelection();
              if (sel && sel.rangeCount > 0) {
                const range = sel.getRangeAt(0);
                const span = document.createElement("span");
                span.className = "html-small";
                span.appendChild(range.extractContents());
                range.insertNode(span);
                if (visual_editor) text = fromHtml(visual_editor.innerHTML);
              }
            }}
            class="fnt-small"
            aria-label="Smaller Font"
          ></button>
          <button
            onclick={() => {
              const sel = window.getSelection();
              if (sel && sel.rangeCount > 0) {
                const range = sel.getRangeAt(0);
                const span = document.createElement("span");
                span.className = "html-big";
                span.appendChild(range.extractContents());
                range.insertNode(span);
                if (visual_editor) text = fromHtml(visual_editor.innerHTML);
              }
            }}
            class="fnt-big"
            aria-label="Bigger Font"
          ></button>
          <button
            onclick={() => {
              const sel = window.getSelection();
              if (sel && sel.rangeCount > 0) {
                const range = sel.getRangeAt(0);
                const span = document.createElement("span");
                span.className = "html-barcode";
                span.appendChild(range.extractContents());
                range.insertNode(span);
                if (visual_editor) text = fromHtml(visual_editor.innerHTML);
              }
            }}
            class="barcode"
            aria-label="Barcode"
          ></button>
        {:else}
          <button
            onclick={() => setAttribute("*")}
            class="bold"
            aria-label="Bold Text"
          ></button>
          <button
            onclick={() => setAttribute("_")}
            class="italic"
            aria-label="Italic Text"
          ></button>
          <button
            onclick={() => setAttribute("__", "__")}
            class="fnt-small"
            aria-label="Smaller Font"
          ></button>
          <button
            onclick={() => setAttribute("^^", "^^")}
            class="fnt-big"
            aria-label="Bigger Font"
          ></button>
          <button
            onclick={() => setAttribute("[|", "|]")}
            class="barcode"
            aria-label="Barcode"
          ></button>
        {/if}

        <button
          class="smile"
          aria-pressed={show_emoji}
          aria-label="Show Emoji Selector"
          onclick={() => {
            show_emoji = !show_emoji;
          }}
        ></button>

        <div class="separator"></div>

        <button
          onclick={() => (horizontal_align = "left")}
          class="align-left"
          aria-pressed={horizontal_align === "left"}
          aria-label="Align Left"
        ></button>
        <button
          onclick={() => (horizontal_align = "center")}
          class="align-center"
          aria-pressed={horizontal_align === "center"}
          aria-label="Align Center"
        ></button>
        <button
          onclick={() => (horizontal_align = "right")}
          class="align-right"
          aria-pressed={horizontal_align === "right"}
          aria-label="Align Right"
        ></button>

        <div class="separator"></div>

        <button
          onclick={() => (vertical_align = "top")}
          class="align-top"
          aria-pressed={vertical_align === "top"}
          aria-label="Align Top"
        ></button>
        <button
          onclick={() => (vertical_align = "middle")}
          class="align-middle"
          aria-pressed={vertical_align === "middle"}
          aria-label="Align Middle"
        ></button>
        <button
          onclick={() => (vertical_align = "bottom")}
          class="align-bottom"
          aria-pressed={vertical_align === "bottom"}
          aria-label="Align Bottom"
        ></button>
      </div>

      {#if show_emoji}
        <Emoji
          onselect={(emoji: string) => {
            if (editor_mode === "visual" && visual_editor) {
              document.execCommand("insertText", false, emoji);
              text = fromHtml(visual_editor.innerHTML);
            } else {
              insertText(emoji);
            }
          }}
        />
      {/if}

      {#if editor_mode === "visual"}
        <div
          bind:this={visual_editor}
          contenteditable="true"
          class="visual-editor"
          oninput={(e) => (text = fromHtml(e.currentTarget.innerHTML))}
        >
          {@html toHtml(text)}
        </div>
      {:else}
        <textarea
          bind:this={textarea}
          bind:value={text}
          placeholder="Write text here..."
        ></textarea>
      {/if}
    </div>
    <details>
      <summary>⚙️ Configuration</summary>
      <div class="config">
        <label>
          Label Width (mm): <input
            type="number"
            bind:value={label_width_mm}
            min="1"
            max="150"
            step="0.1"
          />
        </label>
        <label>
          Label Height (mm): <input
            type="number"
            bind:value={label_height_mm}
            min="1"
            max="18"
            step="0.1"
          />
        </label>
        <label>
          Margin Width (mm): <input
            type="number"
            bind:value={margin_width_mm}
            min="0"
            max="10"
            step="0.1"
          />
        </label>
        <label>
          Margin Height (mm): <input
            type="number"
            bind:value={margin_height_mm}
            min="0"
            max="10"
            step="0.1"
          />
        </label>
        <label>
          <input type="checkbox" bind:checked={auto_font_size} />
          Automatic Font Size
        </label>
        <label>
          <input type="checkbox" bind:checked={search_all_devices} />
          Show All Bluetooth Devices (Troubleshooting)
        </label>
        <p style="font-size: 0.8em; color: #666; margin: 0;">
          Tip: Ensure the printer is <strong>unpaired/disconnected</strong> from your system's Bluetooth settings before searching.
        </p>
        <label>
          Font Size:
          <input
            type="range"
            min="8"
            max="144"
            step="1"
            bind:value={font_size}
            disabled={auto_font_size}
            list="markers"
          />
          <datalist id="markers">
            <option value="12"></option>
            <option value="24"></option>
            <option value="32"></option>
            <option value="48"></option>
            <option value="64"></option>
            <option value="96"></option>
            <option value="128"></option>
          </datalist>
          <span>{font_size.toFixed(0)}</span>
        </label>
      </div>
    </details>
    <div class="actions">
      <button onclick={selectDevice} class="connect-btn">
        {selected_device?.name ? `Connected to ${selected_device.name}` : "Connect Printer"}
      </button>
      <button onclick={printLabel} class="print-btn">Print Label</button>
    </div>
    <div class="l">
      <canvas bind:this={canvas} width={label_width} height={label_height}>
      </canvas>
      <span>
        Label size:
        {(label_width * 0.125).toFixed(1)}
        ×
        {(label_height * 0.125).toFixed(1)}
        mm
      </span>
      <span>
        Margin:
        {(margin_x * 0.125).toFixed(1)}
        ×
        {(margin_y * 0.125).toFixed(1)}
        mm
      </span>
    </div>
  </div>
</main>

<style>
  h1 {
    font-size: 3.2em;
    line-height: 1.1;
    border-bottom: 1px solid #444;
  }
  div.bar {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    justify-content: start;
    column-gap: 8px;
    width: 100%;
  }
  div.r {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    gap: 2em;
  }
  .r > * {
    display: block;
  }
  .l {
    width: 100%;
  }
  .l > * {
    display: block;
    width: 100%;
  }
  .l span {
    color: #888;
    text-align: left;
  }
  textarea,
  .visual-editor,
  button,
  canvas {
    box-sizing: border-box;
    width: 100%;
    border: 1px solid #ccc;
    border-radius: 12px;
  }
  .editor-container {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0;
    border: 1px solid #ccc;
    border-radius: 12px;
    overflow: hidden;
  }
  .editor-tabs {
    display: flex;
    background-color: #eee;
    border-bottom: 1px solid #ccc;
  }
  .tab-btn {
    flex: 1;
    border: none;
    border-radius: 0;
    padding: 8px;
    font-size: 0.9em;
    background-color: transparent;
  }
  .tab-btn[aria-pressed="true"] {
    background-color: #fff;
    font-weight: bold;
  }
  .editor-container .bar {
    border: none;
    border-bottom: 1px solid #eee;
    padding: 8px;
    background-color: #f9f9f9;
    border-radius: 0;
  }
  textarea,
  .visual-editor {
    border: none;
    border-radius: 0;
    min-height: 150px;
    max-height: 300px;
    overflow-y: auto;
    font-size: 24px;
    padding: 12px;
    background-color: #fff;
    text-align: left;
  }
  textarea {
    field-sizing: content;
    min-height: 1lh;
    max-height: 10lh;
    overflow: hidden;
    resize: none;
    background-color: #ccf;
    transition: background-color 0.25s ease-in-out;
  }
  .visual-editor:focus {
    outline: none;
    background-color: #fdfdfd;
  }
  textarea:hover {
    background-color: #ddf;
  }
  :global(.html-barcode) {
    background-color: #ffe0b2;
    font-family: monospace;
    padding: 0 4px;
    border-radius: 4px;
    border: 1px dashed #f57c00;
  }
  :global(.html-big) {
    font-size: 1.4em;
    color: #2e7d32;
    text-decoration: underline;
  }
  :global(.html-small) {
    font-size: 0.7em;
    color: #1565c0;
  }
  canvas {
    border-radius: 32px;
  }
  button {
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    background-color: #ddd;
    cursor: pointer;
    transition: background-color 0.25s ease-in-out;
  }
  .actions {
    display: flex;
    gap: 1em;
    width: 100%;
  }
  .actions button {
    flex: 1;
  }
  .connect-btn {
    background-color: #f0f0f0;
    color: #333;
  }
  .print-btn {
    background-color: #4caf50;
    color: white;
    border-color: #45a049;
  }
  .print-btn:hover {
    background-color: #45a049;
  }
  .bar button:hover,
  button:hover {
    background-color: #ccc;
  }
  details {
    width: 100%;
    box-sizing: border-box;
  }
  .config {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
    padding: 1em;
    border: 2px solid #ddd;
    border-radius: 12px;
    background-color: #f9f9f9;
  }
  .config label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: 500;
  }
  .config input {
    width: 80px;
    padding: 0.3em;
    border: 1px solid #ccc;
    border-radius: 6px;
    text-align: center;
  }
  .config input[type="checkbox"] {
    width: auto;
    margin-right: 0.5em;
  }
  .config input[type="range"] {
    width: calc(100% - 10em);
  }
  .config label:has(input[type="checkbox"]) {
    justify-content: flex-start;
  }
  .bar button {
    padding: 16px;
    background-position-x: center;
    background-position-y: center;
    background-color: #ddd;
    max-width: 28px;
    border-radius: 6px;
  }
  .bar button[aria-pressed="true"] {
    background-color: #def;
  }
  .italic {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWl0YWxpYy1pY29uIGx1Y2lkZS1pdGFsaWMiPjxsaW5lIHgxPSIxOSIgeDI9IjEwIiB5MT0iNCIgeTI9IjQiLz48bGluZSB4MT0iMTQiIHgyPSI1IiB5MT0iMjAiIHkyPSIyMCIvPjxsaW5lIHgxPSIxNSIgeDI9IjkiIHkxPSI0IiB5Mj0iMjAiLz48L3N2Zz4=")
      no-repeat;
  }
  .bold {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWJvbGQtaWNvbiBsdWNpZGUtYm9sZCI+PHBhdGggZD0iTTYgMTJoOWE0IDQgMCAwIDEgMCA4SDdhMSAxIDAgMCAxLTEtMVY1YTEgMSAwIDAgMSAxLTFoN2E0IDQgMCAwIDEgMCA4Ii8+PC9zdmc+")
      no-repeat;
  }
  .smile {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLXNtaWxlLWljb24gbHVjaWRlLXNtaWxlIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxMCIvPjxwYXRoIGQ9Ik04IDE0czEuNSAyIDQgMiA0LTIgNC0yIi8+PGxpbmUgeDE9IjkiIHgyPSI5LjAxIiB5MT0iOSIgeTI9IjkiLz48bGluZSB4MT0iMTUiIHgyPSIxNS4wMSIgeTE9IjkiIHkyPSI5Ii8+PC9zdmc+")
      no-repeat;
  }
  .barcode {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWJhcmNvZGUtaWNvbiBsdWNpZGUtYmFyY29kZSI+PHBhdGggZD0iTTMgNXYxNCIvPjxwYXRoIGQ9Ik04IDV2MTQiLz48cGF0aCBkPSJNMTIgNXYxNCIvPjxwYXRoIGQ9Ik0xNyA1djE0Ii8+PHBhdGggZD0iTTIxIDV2MTQiLz48L3N2Zz4=")
      no-repeat;
  }
  .fnt-small {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWFhcnJvdy1kb3duLWljb24gbHVjaWRlLWEtYXJyb3ctZG93biI+PHBhdGggZD0ibTE0IDEyIDQgNCA0LTQiLz48cGF0aCBkPSJNMTggMTZWNyIvPjxwYXRoIGQ9Im0yIDE2IDQuMDM5LTkuNjlhLjUuNSAwIDAgMSAuOTIzIDBMMTEgMTYiLz48cGF0aCBkPSJNMy4zMDQgMTNoNi4zOTIiLz48L3N2Zz4=")
      no-repeat;
  }
  .fnt-big {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWFhcnJvdy11cC1pY29uIGx1Y2lkZS1hLWFycm93LXVwIj48cGF0aCBkPSJtMTQgMTEgNC00IDQgNCIvPjxwYXRoIGQ9Ik0xOCAxNlY3Ii8+PHBhdGggZD0ibTIgMTYgNC4wMzktOS42OWEuNS41IDAgMCAxIC45MjMgMEwxMSAxNiIvPjxwYXRoIGQ9Ik0zLjMwNCAxM2g2LjM5MiIvPjwvc3ZnPg==")
      no-repeat;
  }
  .align-left {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWFsaWduLWxlZnQiPjxsaW5lIHgxPSIxNyIgeDI9IjMiIHkxPSI2IiB5Mj0iNiIvPjxsaW5lIHgxPSIxOSIgeDI9IjMiIHkxPSIxMiIgeTI9IjEyIi8+PGxpbmUgeDE9IjExIiB4Mj0iMyIgeTE9IjE4IiB5Mj0iMTgiLz48L3N2Zz4=")
      no-repeat;
  }
  .align-center {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWFsaWduLWNlbnRlciI+PGxpbmUgeDE9IjIxIiB4Mj0iMyIgeTE9IjYiIHkyPSI2Ii8+PGxpbmUgeDE9IjE3IiB4Mj0iNyIgeTE9IjEyIiB5Mj0iMTIiLz48bGluZSB4MT0iMTkiIHgyPSI1IiB5MT0iMTgiIHkyPSIxOCIvPjwvc3ZnPg==")
      no-repeat;
  }
  .align-right {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWFsaWduLXJpZ2h0Ij48bGluZSB4MT0iMjEiIHgyPSI3IiB5MT0iNiIgeTI9IjYiLz48bGluZSB4MT0iMjEiIHgyPSIzIiB5MT0iMTIiIHkyPSIxMiIvPjxsaW5lIHgxPSIyMSIgeDI9IjEzIiB5MT0iMTgiIHkyPSIxOCIvPjwvc3ZnPg==")
      no-repeat;
  }
  .align-top {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLWJvdHRvbS1wYW5lbCI+PHJlY3Qgd2lkdGg9IjIwIiBoZWlnaHQ9IjE2IiB4PSIyIiB5PSIzIiByeD0iMiIvPjxsaW5lIHgxPSIyIiB4Mj0iMjIiIHkxPSI3IiB5Mj0iNyIvPjwvc3ZnPg==")
      no-repeat;
  }
  .align-middle {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLW1pZGRsZS1wYW5lbCI+PHJlY3Qgd2lkdGg9IjIwIiBoZWlnaHQ9IjE2IiB4PSIyIiB5PSIzIiByeD0iMiIvPjxsaW5lIHgxPSIyIiB4Mj0iMjIiIHkxPSIxMSIgeTI9IjExIi8+PC9zdmc+")
      no-repeat;
  }
  .align-bottom {
    background: url("data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9ImN1cnJlbnRDb2xvciIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIGNsYXNzPSJsdWNpZGUgbHVjaWRlLXRvcC1wYW5lbCI+PHJlY3Qgd2lkdGg9IjIwIiBoZWlnaHQ9IjE2IiB4PSIyIiB5PSIzIiByeD0iMiIvPjxsaW5lIHgxPSIyIiB4Mj0iMjIiIHkxPSIxNSIgeTI9IjE1Ii8+PC9zdmc+")
      no-repeat;
  }
  .separator {
    width: 2px;
    background-color: #ccc;
    margin: 4px 0;
  }
</style>
