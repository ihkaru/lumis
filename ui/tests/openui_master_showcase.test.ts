import { describe, it, expect } from "bun:test";
import { OpenUiParser } from "../src/lib/components/openui/parser";
import { OpenUiEvaluator } from "../src/lib/components/openui/evaluator";

describe("OpenUI Master Showcase (All 56 Components & Primitives)", () => {
  it("parses and evaluates complete master DSL with all 56 components without omission", () => {
    const masterDsl = `
root = Col("Portal Fasilitas & Statistik Nasional 2026", "lg")

$tab = Denah
$mode = Publik
$kategori = Survei
$persetujuan = true
$kuota = 75
$catatan = "Prioritas pelayanan inklusif disabilitas dan lansia"
$tanggal = 2026-10-21
$durasi = 300
$notif = true
$budget = 500000000
$bunga = 0.06
$tenor = 60
$pencarian = "PST BPS"

$anggaranBulanan = @Round(@Payment($budget, $bunga, $tenor))
$sampelNilai = [12, 18, 25, 30, 42, 55, 68]
$totalSampel = @Sum($sampelNilai)
$rataRata = @Round(@Avg($sampelNilai), 1)
$skorMaks = @Max($sampelNilai)
$skorMin = @Min($sampelNilai)
$labelBulan = Jan | Feb | Mar | Apr | Mei | Jun | Jul
$dataTren = 15 | 22 | 28 | 35 | 45 | 58 | 70

Col(
  Title("Portal Terpadu Fasilitas & Operasional Sensus BPS 2026", "xl"),
  Subtitle("Sistem Orkestrasi Layanan Publik, Denah Arsitektur, Simulasi Keuangan, dan Visualisasi Terlengkap"),

  Row(
    Badge("Status Operasional Aktif", success),
    Badge("Verifikasi BPS Resmi", info),
    Badge("Mode $mode", warning),
    Icon(shield, sm, success)
  ),

  Divider(),

  Row(
    Metric("Total Kuota Responden", "$kuota Orang", "Kapasitas harian PST"),
    Metric("Anggaran Bulanan", "Rp $anggaranBulanan", "Cicilan investasi PST"),
    Metric("Rata-Rata Kepuasan", "$rataRata Poin", "Skala 100 dari $totalSampel responden"),
    Metric("Skor Tertinggi", "$skorMaks Poin", "Batas aman min $skorMin")
  ),

  Card(
    Title("Panel Kontrol Interaktif & Form Evaluasi", "lg"),
    Text("Atur parameter operasional secara reaktif untuk memperbarui seluruh tampilan widget di bawah ini."),
    SegmentedControl($tab, Denah | Grafik | Media | Kuis),
    Row(
      Select($kategori, Sensus | Survei | Evaluasi | Publikasi),
      RadioGroup($mode, Publik | Internal | Rahasia),
      DatePicker($tanggal, "Tanggal Operasional")
    ),
    Slider($kuota, 10, 200, 5, "Kapasitas Harian PST", "Orang"),
    Row(
      Input($pencarian, "Cari Modul / Petugas", text, "Ketik nama petugas"),
      Checkbox($persetujuan, "Saya menyetujui SOP Pelayanan Statistik Terpadu")
    ),
    TextArea($catatan, "Catatan Fasilitator PST", 2),
    Row(
      Timer($durasi, "Durasi Sesi Konsultasi"),
      Button("Tambah Durasi", @Increment($durasi, 60)),
      Button("Kurang Durasi", @Decrement($durasi, 30)),
      Button("Toggle Notif", @Toggle($notif)),
      Button("Reset Data", @Reset($kuota)),
      Button("Salin Info", @CopyToClipboard("Sistem PST BPS 2026 Aktif")),
      Button("Tanya Asisten", @ToAssistant("Jelaskan ringkasan fasilitas PST ini"))
    ),
    Form(
      Row(
        Label("Konfirmasi Pengajuan Sesi", true, "idPengajuan"),
        IconButton(check, @Set($persetujuan, true), "Setujui Cepat")
      ),
      @Set($kuota, 100)
    )
  ),

  Grid(
    Box(
      Title("Diagram Struktur & Denah Canvas SVG", "md"),
      Caption("Tata letak fisik ruang layanan publik standar BPS berbasis Canvas vektor murni."),
      Canvas(600, 260,
        Rect(10, 10, 580, 240, #f8fafc),
        Rect(30, 30, 240, 80, #dbeafe),
        Rect(300, 30, 260, 80, #fef3c7),
        Rect(30, 130, 530, 100, #dcfce7),
        Circle(150, 70, 20, #3b82f6),
        Circle(430, 70, 20, #f59e0b),
        Line(30, 120, 560, 120, #94a3b8, 2),
        Path("M 40 180 Q 280 140 520 180", none, #10b981, 3),
        Group([
          SvgText(40, 55, "LOKET KONSULTASI PST", md),
          SvgText(310, 55, "RUANG OLAH DATA", md),
          SvgText(50, 165, "ZONA PELAYANAN UTAMA - 120 METER PERSEGI", sm)
        ])
      )
    ),
    Box(
      Title("Distribusi Layanan & Komposisi Anggaran", "md"),
      DonutChart(Sensus | Survei | Konsultasi | Rekomendasi, 45 | 25 | 20 | 10, "Komposisi Kunjungan PST"),
      PieChart(Operasional | Sarana | Honor | Publikasi, 40 | 30 | 20 | 10, "Proporsi Anggaran 2026")
    )
  ),

  Card(
    Title("Visualisasi Tren Waktu Nyata", "lg"),
    LineChart($labelBulan, $dataTren, "Tren Volume Pengunjung PST 2026"),
    BarChart(Padi | Jagung | Kedelai | Cabai | Bawang, 85 | 62 | 45 | 78 | 54, "Hasil Panen Sensus Pertanian 2026"),
    AreaChart($labelBulan, 10 | 18 | 24 | 30 | 38 | 48 | 62, "Akumulasi Digitalisasi Metadata")
  ),

  Grid(
    Box(
      Title("Peta Wilayah & Media Interaktif", "md"),
      Map(-6.1754, 106.8272, 14, ["BPS Pusat", "Monas", "PST Jakarta"], "Wilayah Kerja BPS RI"),
      Carousel([
        Image("https://images.unsplash.com/photo-1517048676732-d65bc937f952", "Gedung Kantor PST", 400, 200),
        Image("https://images.unsplash.com/photo-1497366216548-37526070297c", "Ruang Rapat Sensus", 400, 200)
      ]),
      Video("https://www.w3schools.com/html/mov_bbb.mp4", "https://images.unsplash.com/photo-1517048676732-d65bc937f952")
    ),
    Box(
      Title("Dokumentasi, Referensi & Pembelajaran", "md"),
      FlashCard("Konsep IHK", "Indeks Harga Konsumen adalah indikator pengukur rata-rata perubahan harga barang dan jasa", false),
      Confetti(true, 4000),
      Audio("https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3", false, false),
      Popover("Detail Regulasi", "Standar Pelayanan Publik Mengacu pada UU No. 25 Tahun 2009 tentang Pelayanan Publik"),
      Pressable(
        Row(
          Icon(info, sm, primary),
          Text("Tekan tombol ini untuk verifikasi kartu kendali pelayanan."),
          Badge(Prioritas, danger)
        ),
        @Set($tab, Kuis)
      )
    )
  ),

  Card(
    Title("Metadata, Format Rumus & Data Tabular", "lg"),
    Quote("Statistik yang akurat adalah kompas bagi kebijakan bangsa yang adil dan beradab.", "Kepala Badan Pusat Statistik"),
    Math("IHK = (Sum(Pn * W) / Sum(P0 * W)) * 100"),
    Code("SELECT id, nama_layanan, kuota FROM layanan_pst WHERE status = active;", "sql"),
    Markdown("PST BPS menyediakan **Satu Data Indonesia** dengan prinsip interoperabilitas data, standardisasi metadata, dan open standard format."),
    List([
      ListItem("Survei Sosial Ekonomi Nasional - Maret 2026"),
      ListItem("Survei Angkatan Kerja Nasional - Agustus 2026"),
      ListItem("Pembaruan Direktori Usaha Ekonomi - Tahunan")
    ], true),
    Table(
      ["Kode", "Nama Wilayah", "Target Sensus", "Progres Capaian", "Tingkat Keandalan"],
      [
        ["3171", "Jakarta Pusat", "125000", "98.4%", "Sangat Tinggi"],
        ["3273", "Kota Bandung", "210000", "95.1%", "Tinggi"],
        ["3578", "Kota Surabaya", "280000", "97.8%", "Sangat Tinggi"],
        ["5171", "Kota Denpasar", "95000", "94.2%", "Baik"]
      ]
    ),
    Link("Kunjungi Portal Resmi Satu Data Indonesia BPS", "https://bps.go.id"),
    Spacer()
  )
)
`;

    const parser = new OpenUiParser();
    const prog = parser.parseProgram(masterDsl);
    expect(prog.statements.length).toBe(23);

    const ctx = {
      state: {
        tab: "Denah",
        mode: "Publik",
        kategori: "Survei",
        persetujuan: true,
        kuota: 75,
        catatan: "Prioritas pelayanan",
        tanggal: "2026-10-21",
        durasi: 300,
        notif: true,
        budget: 500000000,
        bunga: 0.06,
        tenor: 60,
        pencarian: "PST BPS",
      },
      initialState: {},
      definitions: new Map(),
      handlers: {},
      setState: () => {},
      onToAssistant: () => {},
    };

    const evaluator = new OpenUiEvaluator();
    const result = evaluator.evaluateProgram(prog, ctx);
    expect(result).not.toBeNull();
    expect(result?.type).toBe("Col");
    expect(result?.children?.length).toBe(10);

    const allUsedTypes = new Set<string>();
    function collectAllTypes(node: any) {
      if (!node) return;
      if (typeof node === "object" && node.type) {
        allUsedTypes.add(node.type);
        if (Array.isArray(node.children)) {
          node.children.forEach(collectAllTypes);
        }
        if (Array.isArray(node.props?.items)) {
          node.props.items.forEach(collectAllTypes);
        }
      }
    }
    collectAllTypes(result);

    // Exactly all 56 components verified
    expect(allUsedTypes.size).toBe(56);
  });
});
