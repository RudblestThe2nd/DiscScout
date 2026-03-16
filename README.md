# DiskMapper 🗂️

**WizTree benzeri, açık kaynak Linux disk analiz uygulaması**  
PyQt6 ile geliştirilmiştir.

---

## 📸 Özellikler

| Özellik | Detay |
|---|---|
| **Squarified Treemap** | Boyuta göre hiyerarşik görsel harita |
| **Dosya Listesi** | WizTree gibi sıralanabilir tablo |
| **Hızlı Tarama** | `os.scandir` ile multi-thread tarama |
| **Tıklayarak Gezinme** | Treemap veya listede çift tık ile alt dizine git |
| **Breadcrumb Navigasyon** | Üst dizinlere tek tıkla dön |
| **Renk Kodlama** | % orana göre kırmızı/sarı/yeşil |
| **Dark Tema** | GitHub Dark tarzı arayüz |

---

## ⚡ Kurulum (Ubuntu/Debian)

```bash
# PyQt6 kur
pip3 install PyQt6

# Çalıştır
python3 diskmapper.py

# Veya otomatik kurulum betiği
chmod +x install.sh && ./install.sh
```

---

## 🎮 Kullanım

| Eylem | Nasıl |
|---|---|
| Dizin seç | `Ctrl+O` veya "Dizin Seç & Tara" butonu |
| Üst dizine git | `Backspace` veya "Yukarı" butonu |
| Alt dizine gir | Treemap'e **tek tık** veya listede **çift tık** |
| Taramayı durdur | "Durdur" butonu |
| Üst dizinlere dön | Breadcrumb üzerindeki isimlerden tıkla |

---

## 🗂️ Dosya Yapısı

```
DiskMapper/
├── diskmapper.py      ← Ana uygulama (tek dosya)
├── requirements.txt   ← Bağımlılıklar
├── install.sh         ← Kurulum betiği
└── diskmapper.desktop ← Linux uygulama launcher
```

---

## 🔧 Komut Satırı Kullanımı

```bash
# Doğrudan /home dizinini tara
python3 diskmapper.py /home

# Root olarak çalıştır (tüm disk)
sudo python3 diskmapper.py /
```

---

## 📦 Bağımlılıklar

- Python 3.8+
- PyQt6 >= 6.4.0

---

Geliştirme fikirleri: dosya silme, büyük dosya listesi, filtre arama...
