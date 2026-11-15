"""Builds the Windows OQ test protocol (Excel) from the end-to-end guide's steps.

    python scripts/make-oq.py <steps.json> <output.xlsx>

steps.json comes from tests/manual/E2E-windows.md, each check split into action and
expected result. The workbook is for Carlos to fill in: Portada (protocol data, scope,
instructions, signatures), Pruebas (one row per step), Resumen (formulas per block) and
Desviaciones (the log).
"""

import json
import sys

from openpyxl import Workbook
from openpyxl.formatting.rule import CellIsRule
from openpyxl.styles import Alignment, Border, Font, PatternFill, Side
from openpyxl.utils import get_column_letter
from openpyxl.worksheet.datavalidation import DataValidation

SRC, OUT = sys.argv[1], sys.argv[2]
data = json.load(open(SRC, encoding="utf-8"))

VERSION = "0.2.4"
BUILD = "8bbb93f"
DOC_ID = "XS-OQ-WIN-001"
DATE = "2026-11-15"

# Carlos's spreadsheet style: Arial, green theme.
DARK = "1B4332"
CAT = "2D6A4F"
SESSION = "1B4332"
HEAD = "B7E4C7"
INPUT = "FFFDE7"
GRAY = "F2F2F2"
F = "Arial"


def font(size=10, bold=False, color=None, italic=False):
    return Font(name=F, size=size, bold=bold, color=color, italic=italic)


def fill(hex_):
    return PatternFill("solid", start_color=hex_, end_color=hex_)


thin = Side(style="thin", color="CCCCCC")
BOX = Border(left=thin, right=thin, top=thin, bottom=thin)
WRAP = Alignment(wrap_text=True, vertical="top")
CENTER = Alignment(horizontal="center", vertical="top", wrap_text=True)


def footer(ws, landscape=True):
    ws.oddFooter.left.text = "Confidencial"
    ws.oddFooter.left.font = F
    ws.oddFooter.right.text = "Página &P de &N"
    ws.oddFooter.right.font = F
    ws.oddHeader.right.text = f"XcreenShot | {DOC_ID} v1.0"
    ws.oddHeader.right.font = F
    ws.page_setup.orientation = "landscape" if landscape else "portrait"
    ws.page_setup.paperSize = ws.PAPERSIZE_LETTER
    ws.page_setup.fitToWidth = 1
    ws.page_setup.fitToHeight = 0
    ws.sheet_properties.pageSetUpPr.fitToPage = True


wb = Workbook()

# --------------------------------------------------------------------------- Portada
ws = wb.active
ws.title = "Portada"
ws.sheet_view.showGridLines = False
ws.column_dimensions["A"].width = 30
ws.column_dimensions["B"].width = 95
r = 1
ws.cell(r, 1, "Protocolo de Calificación Operacional (OQ)").font = font(16, True, DARK)
r += 1
ws.cell(r, 1, "XcreenShot para Windows: prueba de punta a punta").font = font(12, True, CAT)
r += 2

meta = [
    ("Documento", DOC_ID),
    ("Versión del protocolo", "1.0"),
    ("Sistema", "XcreenShot para Windows (captura, anotación y grabación de pantalla)"),
    ("Versión del software", f"{VERSION} (compilación {BUILD})"),
    ("Plataforma", "Windows 11 (x64)"),
    ("Responsable", "Carlos Acevedo"),
    ("Preparado por", "Claude (asistente de desarrollo), a partir de tests/manual/E2E-windows.md"),
    ("Fecha", DATE),
    ("Estado", "Listo para ejecución"),
    ("Clasificación", "Confidencial"),
]
for k, v in meta:
    a = ws.cell(r, 1, k)
    a.font = font(10, True)
    a.fill = fill(HEAD)
    a.border = BOX
    b = ws.cell(r, 2, v)
    b.font = font(10)
    b.border = BOX
    b.alignment = WRAP
    r += 1
r += 1


def section(title):
    global r
    ws.merge_cells(start_row=r, start_column=1, end_row=r, end_column=2)
    c = ws.cell(r, 1, title)
    c.font = font(11, True, "FFFFFF")
    c.fill = fill(CAT)
    r += 1


def para(text, label=""):
    global r
    if label:
        ws.cell(r, 1, label).font = font(10, True)
        ws.cell(r, 1).alignment = WRAP
    c = ws.cell(r, 2, text)
    c.font = font(10)
    c.alignment = WRAP
    r += 1


def bullets(items, prefix="•"):
    for i, t in enumerate(items, 1):
        para(t, f"{prefix} {i}" if prefix != "•" else "•")


section("1. Objetivo")
para(
    "Verificar que XcreenShot para Windows, versión instalada "
    f"{VERSION}, funciona como se especifica en su uso real: instalación y actualización, "
    "captura, editor, biblioteca y bandeja, funciones inteligentes, video, compartir y "
    "automatización. Cada caso cita los requisitos del spec (IDs) que comprueba."
)
r += 1
section("2. Alcance")
para(
    "Funciones construidas en Windows y aún no verificadas por el responsable, en tres "
    "sesiones de unos 90 minutos. Lo ya verificado antes se indica en cada bloque. Los casos "
    "detallados de cada función están en las guías de tests/manual/ que cita cada bloque."
)
r += 1
section("3. Fuera de alcance")
bullets(data.get("out_of_scope", []))
r += 1
section("4. Prerrequisitos")
bullets([f"XcreenShot {VERSION} instalado (Ayuda > Acerca de)."] + data.get("prerequisites", []))
r += 1
section("5. Seguridad durante la ejecución")
bullets(data.get("safety", []))
r += 1
section("6. Instrucciones de ejecución")
bullets(
    [
        "Ejecutar los casos en el orden de la hoja Pruebas, una sesión cada vez.",
        "Rellenar solo las columnas con fondo amarillo claro: Resultado obtenido, Resultado, "
        "Ejecutado por, Fecha, Evidencia, Desviación y Comentarios.",
        "Resultado: PASS si lo obtenido coincide con lo esperado; FAIL si no; N/A si el caso no "
        "aplica (por ejemplo, requiere hardware que no hay). Un N/A lleva su motivo en Comentarios.",
        "Evidencia: nombre del archivo de la captura o del video (recomendada en todo FAIL).",
        "Todo FAIL abre una desviación en la hoja Desviaciones (DEV-001, DEV-002…) y se anota su ID "
        "en la columna Desviación.",
        "La columna Condición indica casos que solo se ejecutan si se cumple (hardware, clave, "
        "segundo equipo). Si no se cumple: N/A.",
        "El caso 1.1.1 ya está registrado como ejemplo de cómo se rellena una fila.",
    ]
)
r += 1
section("7. Criterios de aceptación")
bullets(
    [
        "Todos los casos ejecutados tienen Resultado PASS o N/A justificado.",
        "Ninguna desviación Crítica o Mayor abierta. Las Menores tienen acción correctiva acordada.",
        "La hoja Resumen muestra cada bloque como Completo.",
    ]
)
r += 1
section("8. Pruebas que necesitan algo especial")
ws.cell(r, 1, "Necesita").font = font(10, True)
ws.cell(r, 1).fill = fill(HEAD)
ws.cell(r, 2, "Qué se prueba (guía)").font = font(10, True)
ws.cell(r, 2).fill = fill(HEAD)
r += 1
for s in data.get("special", []):
    ws.cell(r, 1, s.get("need", "")).font = font(10)
    ws.cell(r, 1).alignment = WRAP
    g = s.get("guides", "")
    ws.cell(r, 2, s.get("what", "") + (f" ({g})" if g else "")).font = font(10)
    ws.cell(r, 2).alignment = WRAP
    for col in (1, 2):
        ws.cell(r, col).border = BOX
    r += 1
r += 1
section("9. Aprobaciones")
ws.column_dimensions["C"].width = 22
ws.column_dimensions["D"].width = 16
for i, h in enumerate(["Rol", "Nombre", "Firma", "Fecha"], 1):
    c = ws.cell(r, i, h)
    c.font = font(10, True)
    c.fill = fill(HEAD)
    c.border = BOX
r += 1
for role in ["Ejecutado por", "Revisado por", "Aprobado por"]:
    ws.cell(r, 1, role).font = font(10, True)
    for i in range(1, 5):
        ws.cell(r, i).border = BOX
        if i > 1:
            ws.cell(r, i).fill = fill(INPUT)
    ws.row_dimensions[r].height = 28
    r += 1
footer(ws, landscape=False)

# --------------------------------------------------------------------------- Pruebas
ws = wb.create_sheet("Pruebas")
ws.sheet_view.showGridLines = False
cols = [
    ("ID", 9),
    ("Requisitos", 18),
    ("Acción", 48),
    ("Resultado esperado", 48),
    ("Condición", 16),
    ("Resultado obtenido", 34),
    ("Resultado", 11),
    ("Ejecutado por", 11),
    ("Fecha", 12),
    ("Evidencia", 18),
    ("Desviación", 11),
    ("Comentarios", 30),
    ("Bloque", 8),
]
for i, (_, w) in enumerate(cols, 1):
    ws.column_dimensions[get_column_letter(i)].width = w
ws.column_dimensions["M"].hidden = True
NCOL = len(cols)

ws.cell(1, 1, f"{DOC_ID} · Casos de prueba · XcreenShot {VERSION}").font = font(14, True, DARK)
ws.cell(2, 1, "Rellena solo las columnas amarillas. Resultado: elige PASS, FAIL o N/A. Todo FAIL lleva su desviación (hoja Desviaciones).").font = font(9, color="555555")
HROW = 4
for i, (h, _) in enumerate(cols, 1):
    c = ws.cell(HROW, i, h)
    c.font = font(10, True)
    c.fill = fill(HEAD)
    c.border = BOX
    c.alignment = CENTER
ws.freeze_panes = ws.cell(HROW + 1, 1)
ws.print_title_rows = f"{HROW}:{HROW}"

dv = DataValidation(type="list", formula1='"PASS,FAIL,N/A"', allow_blank=True)
dv.error = "Elige PASS, FAIL o N/A."
dv.errorTitle = "Resultado"
dv.prompt = "PASS, FAIL o N/A"
ws.add_data_validation(dv)

row = HROW + 1
blocks_index = []  # (block id, title, session id)
step_rows = []
for s in data["sessions"]:
    ws.merge_cells(start_row=row, start_column=1, end_row=row, end_column=NCOL - 1)
    c = ws.cell(row, 1, f"Sesión {s['id']} · {s['title']} (unos {s['minutes']} min)")
    c.font = font(12, True, "FFFFFF")
    c.fill = fill(SESSION)
    ws.row_dimensions[row].height = 20
    row += 1
    for b in s["blocks"]:
        ws.merge_cells(start_row=row, start_column=1, end_row=row, end_column=NCOL - 1)
        text = f"{b['id']} {b['title']} ({b['minutes']} min) — {b['purpose']}"
        if b.get("guides"):
            text += f"   Guías: {b['guides']}"
        if b.get("note"):
            text += f"   Nota: {b['note']}"
        c = ws.cell(row, 1, text)
        c.font = font(10, True, "FFFFFF")
        c.fill = fill(CAT)
        c.alignment = Alignment(wrap_text=True, vertical="center")
        ws.row_dimensions[row].height = 30
        row += 1
        blocks_index.append((b["id"], b["title"], s["id"]))
        for n, st in enumerate(b["steps"], 1):
            sid = f"{b['id']}.{n}"
            values = [
                sid,
                b.get("requirements", ""),
                st["action"],
                st["expected"],
                st.get("conditional", ""),
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "B" + b["id"],  # a text key: "3.1" and "3.10" are one number to COUNTIFS
            ]
            pre = st.get("prefilled", "")
            if pre.startswith("PASS"):
                values[5] = "Volvió con su ventana y el aviso \"XcreenShot se actualizó\"."
                values[6] = "PASS"
                values[7] = "CA"
                values[8] = pre.split(" ", 1)[1] if " " in pre else ""
            for i, v in enumerate(values, 1):
                c = ws.cell(row, i, v)
                c.font = font(10, bold=(i == 1))
                c.alignment = CENTER if i in (1, 7, 8, 9, 11) else WRAP
                c.border = BOX
                if 6 <= i <= 12:
                    c.fill = fill(INPUT)
            dv.add(ws.cell(row, 7))
            step_rows.append(row)
            row += 1
last = row - 1
rng = f"G{HROW + 1}:G{last}"
ws.conditional_formatting.add(rng, CellIsRule(operator="equal", formula=['"PASS"'], font=Font(name=F, bold=True, color="00B050", size=11)))
ws.conditional_formatting.add(rng, CellIsRule(operator="equal", formula=['"FAIL"'], font=Font(name=F, bold=True, color="FF0000", size=11)))
ws.conditional_formatting.add(rng, CellIsRule(operator="equal", formula=['"N/A"'], font=Font(name=F, bold=True, color="808080", size=11)))
footer(ws)

# --------------------------------------------------------------------------- Resumen
ws = wb.create_sheet("Resumen", 1)
ws.sheet_view.showGridLines = False
ws.cell(1, 1, f"{DOC_ID} · Resumen por bloque").font = font(14, True, DARK)
ws.cell(2, 1, "Se calcula solo a partir de la hoja Pruebas: no hace falta escribir aquí.").font = font(9, color="555555")
heads = [("Bloque", 9), ("Título", 44), ("Sesión", 9), ("Casos", 9), ("PASS", 9), ("FAIL", 9), ("N/A", 9), ("Pendientes", 11), ("Estado", 14)]
for i, (h, w) in enumerate(heads, 1):
    ws.column_dimensions[get_column_letter(i)].width = w
    c = ws.cell(4, i, h)
    c.font = font(10, True)
    c.fill = fill(HEAD)
    c.border = BOX
    c.alignment = CENTER
P = f"Pruebas!$M${HROW + 1}:$M${last}"
G = f"Pruebas!$G${HROW + 1}:$G${last}"
rr = 5
for bid, title, sid in blocks_index:
    vals = [
        bid,
        title,
        sid,
        f'=COUNTIFS({P},"B"&A{rr})',
        f'=COUNTIFS({P},"B"&A{rr},{G},"PASS")',
        f'=COUNTIFS({P},"B"&A{rr},{G},"FAIL")',
        f'=COUNTIFS({P},"B"&A{rr},{G},"N/A")',
        f"=D{rr}-E{rr}-F{rr}-G{rr}",
        f'=IF(F{rr}>0,"Con fallos",IF(H{rr}=0,"Completo",IF(H{rr}=D{rr},"Sin empezar","En curso")))',
    ]
    for i, v in enumerate(vals, 1):
        c = ws.cell(rr, i, v)
        c.font = font(10, bold=(i == 1))
        c.border = BOX
        c.alignment = CENTER if i != 2 else WRAP
        if rr % 2 == 0:
            c.fill = fill(GRAY)
    rr += 1
tot = rr
ws.cell(tot, 1, "Total").font = font(10, True)
ws.cell(tot, 2, "").border = BOX
for col in "DEFGH":
    c = ws[f"{col}{tot}"]
    c.value = f"=SUM({col}5:{col}{tot - 1})"
    c.font = font(10, True)
for i in range(1, 10):
    ws.cell(tot, i).border = BOX
    ws.cell(tot, i).fill = fill(HEAD)
    ws.cell(tot, i).alignment = CENTER
ws.cell(tot + 2, 1, "Avance").font = font(10, True)
c = ws.cell(tot + 2, 2, f"=IF(D{tot}=0,0,(E{tot}+F{tot}+G{tot})/D{tot})")
c.number_format = "0%"
c.font = font(10, True)
c.alignment = Alignment(horizontal="left")
ws.cell(tot + 3, 1, "PASS sobre ejecutados").font = font(10, True)
c = ws.cell(tot + 3, 2, f"=IF((E{tot}+F{tot})=0,0,E{tot}/(E{tot}+F{tot}))")
c.number_format = "0%"
c.font = font(10, True)
c.alignment = Alignment(horizontal="left")
ws.cell(tot + 4, 1, "Desviaciones abiertas").font = font(10, True)
c = ws.cell(tot + 4, 2, '=COUNTIFS(Desviaciones!$I$5:$I$200,"Abierta")')
c.font = font(10, True)
c.alignment = Alignment(horizontal="left")
rng = f"I5:I{tot - 1}"
ws.conditional_formatting.add(rng, CellIsRule(operator="equal", formula=['"Completo"'], font=Font(name=F, bold=True, color="00B050")))
ws.conditional_formatting.add(rng, CellIsRule(operator="equal", formula=['"Con fallos"'], font=Font(name=F, bold=True, color="FF0000")))
ws.freeze_panes = "A5"
footer(ws, landscape=False)

# --------------------------------------------------------------------------- Desviaciones
ws = wb.create_sheet("Desviaciones")
ws.sheet_view.showGridLines = False
ws.cell(1, 1, f"{DOC_ID} · Registro de desviaciones").font = font(14, True, DARK)
ws.cell(2, 1, "Una fila por cada FAIL. La fila DEV-000 es un ejemplo: bórrala cuando registres la primera real.").font = font(9, color="555555")
dheads = [
    ("Desviación", 11),
    ("Caso", 9),
    ("Fecha", 12),
    ("Descripción", 44),
    ("Severidad", 11),
    ("Causa", 30),
    ("Acción correctiva", 34),
    ("Responsable", 14),
    ("Estado", 11),
    ("Fecha de cierre", 13),
    ("Verificación", 26),
]
for i, (h, w) in enumerate(dheads, 1):
    ws.column_dimensions[get_column_letter(i)].width = w
    c = ws.cell(4, i, h)
    c.font = font(10, True)
    c.fill = fill(HEAD)
    c.border = BOX
    c.alignment = CENTER
sev = DataValidation(type="list", formula1='"Crítica,Mayor,Menor"', allow_blank=True)
est = DataValidation(type="list", formula1='"Abierta,Cerrada"', allow_blank=True)
ws.add_data_validation(sev)
ws.add_data_validation(est)
example = [
    "DEV-000",
    "1.4.2",
    DATE,
    "Ejemplo: el Selector de ventanas no muestra las ventanas minimizadas.",
    "Menor",
    "Ejemplo: el filtro excluía las ventanas minimizadas.",
    "Ejemplo: corregido en la versión 0.2.5.",
    "Claude",
    "Cerrada",
    DATE,
    "Ejemplo: caso 1.4.2 repetido con la 0.2.5: PASS.",
]
for rr in range(5, 65):
    for i in range(1, len(dheads) + 1):
        c = ws.cell(rr, i, example[i - 1] if rr == 5 else None)
        c.font = font(10, italic=(rr == 5), color=("808080" if rr == 5 else None))
        c.border = BOX
        c.alignment = WRAP
        c.fill = fill(INPUT)
    sev.add(ws.cell(rr, 5))
    est.add(ws.cell(rr, 9))
ws.freeze_panes = "A5"
footer(ws)

wb.save(OUT)
print(f"{OUT}: {len(step_rows)} steps in {len(blocks_index)} blocks")
