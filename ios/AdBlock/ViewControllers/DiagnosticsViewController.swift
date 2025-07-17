import UIKit

class DiagnosticsViewController: UIViewController {
    
    // MARK: - UI Elements
    
    private let segmentedControl: UISegmentedControl = {
        let items = ["クラッシュレポート", "使用統計"]
        let control = UISegmentedControl(items: items)
        control.selectedSegmentIndex = 0
        control.translatesAutoresizingMaskIntoConstraints = false
        return control
    }()
    
    private let tableView: UITableView = {
        let table = UITableView(frame: .zero, style: .insetGrouped)
        table.translatesAutoresizingMaskIntoConstraints = false
        return table
    }()
    
    private let clearButton: UIButton = {
        let button = UIButton(type: .system)
        button.setTitle("データをクリア", for: .normal)
        button.setTitleColor(.systemRed, for: .normal)
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()
    
    // MARK: - Properties
    
    private var crashReports: [CrashReport] = []
    private var crashStats: CrashStatistics?
    private var analyticsSummary: AnalyticsSummary?
    
    // MARK: - Lifecycle
    
    override func viewDidLoad() {
        super.viewDidLoad()
        
        setupUI()
        loadData()
        
        // Add navigation items
        navigationItem.rightBarButtonItem = UIBarButtonItem(
            image: UIImage(systemName: "gearshape"),
            style: .plain,
            target: self,
            action: #selector(showSettings)
        )
    }
    
    // MARK: - Setup
    
    private func setupUI() {
        view.backgroundColor = .systemBackground
        title = "診断とレポート"
        
        view.addSubview(segmentedControl)
        view.addSubview(tableView)
        view.addSubview(clearButton)
        
        NSLayoutConstraint.activate([
            segmentedControl.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 16),
            segmentedControl.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 16),
            segmentedControl.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -16),
            
            tableView.topAnchor.constraint(equalTo: segmentedControl.bottomAnchor, constant: 16),
            tableView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            tableView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            tableView.bottomAnchor.constraint(equalTo: clearButton.topAnchor, constant: -16),
            
            clearButton.leadingAnchor.constraint(equalTo: view.leadingAnchor, constant: 16),
            clearButton.trailingAnchor.constraint(equalTo: view.trailingAnchor, constant: -16),
            clearButton.bottomAnchor.constraint(equalTo: view.safeAreaLayoutGuide.bottomAnchor, constant: -16),
            clearButton.heightAnchor.constraint(equalToConstant: 44)
        ])
        
        tableView.delegate = self
        tableView.dataSource = self
        tableView.register(UITableViewCell.self, forCellReuseIdentifier: "Cell")
        tableView.register(DiagnosticsDetailCell.self, forCellReuseIdentifier: "DetailCell")
        
        segmentedControl.addTarget(self, action: #selector(segmentChanged), for: .valueChanged)
        clearButton.addTarget(self, action: #selector(clearData), for: .touchUpInside)
    }
    
    private func loadData() {
        // Load crash reports
        crashReports = CrashReporter.shared.getRecentReports(limit: 20)
        crashStats = CrashReporter.shared.getStatistics()
        
        // Load analytics summary
        analyticsSummary = Analytics.shared.getSummary()
        
        tableView.reloadData()
    }
    
    // MARK: - Actions
    
    @objc private func segmentChanged() {
        tableView.reloadData()
        updateClearButton()
    }
    
    @objc private func clearData() {
        let alertController = UIAlertController(
            title: "データをクリア",
            message: segmentedControl.selectedSegmentIndex == 0 ?
                "すべてのクラッシュレポートを削除しますか？" :
                "すべての使用統計を削除しますか？",
            preferredStyle: .alert
        )
        
        alertController.addAction(UIAlertAction(title: "キャンセル", style: .cancel))
        alertController.addAction(UIAlertAction(title: "削除", style: .destructive) { [weak self] _ in
            if self?.segmentedControl.selectedSegmentIndex == 0 {
                CrashReporter.shared.clearAllReports()
            } else {
                Analytics.shared.clearAllData()
            }
            self?.loadData()
        })
        
        present(alertController, animated: true)
    }
    
    @objc private func showSettings() {
        let settingsVC = DiagnosticsSettingsViewController()
        let navController = UINavigationController(rootViewController: settingsVC)
        present(navController, animated: true)
    }
    
    private func updateClearButton() {
        clearButton.isEnabled = segmentedControl.selectedSegmentIndex == 0 ?
            !crashReports.isEmpty :
            (analyticsSummary?.totalEvents ?? 0) > 0
    }
}

// MARK: - UITableViewDataSource

extension DiagnosticsViewController: UITableViewDataSource {
    
    func numberOfSections(in tableView: UITableView) -> Int {
        if segmentedControl.selectedSegmentIndex == 0 {
            // Crash reports: statistics + reports
            return crashReports.isEmpty ? 1 : 2
        } else {
            // Analytics: summary + events by category + metrics
            return 3
        }
    }
    
    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        if segmentedControl.selectedSegmentIndex == 0 {
            // Crash reports
            if section == 0 {
                return 1 // Statistics card
            } else {
                return crashReports.count
            }
        } else {
            // Analytics
            switch section {
            case 0:
                return 1 // Summary
            case 1:
                return analyticsSummary?.eventsByCategory.count ?? 0
            case 2:
                return analyticsSummary?.metrics.count ?? 0
            default:
                return 0
            }
        }
    }
    
    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        if segmentedControl.selectedSegmentIndex == 0 {
            // Crash reports
            if indexPath.section == 0 {
                let cell = tableView.dequeueReusableCell(withIdentifier: "DetailCell", for: indexPath) as! DiagnosticsDetailCell
                cell.configure(with: crashStats)
                return cell
            } else {
                let cell = tableView.dequeueReusableCell(withIdentifier: "Cell", for: indexPath)
                let report = crashReports[indexPath.row]
                
                var content = cell.defaultContentConfiguration()
                content.text = getCrashTypeLabel(report.type)
                content.secondaryText = formatDate(report.timestamp)
                content.image = UIImage(systemName: "exclamationmark.triangle.fill")
                content.imageProperties.tintColor = .systemRed
                
                cell.contentConfiguration = content
                cell.accessoryType = .disclosureIndicator
                
                return cell
            }
        } else {
            // Analytics
            let cell = tableView.dequeueReusableCell(withIdentifier: "Cell", for: indexPath)
            var content = cell.defaultContentConfiguration()
            
            switch indexPath.section {
            case 0:
                // Summary
                content.text = "合計イベント"
                content.secondaryText = "\(analyticsSummary?.totalEvents ?? 0)"
            case 1:
                // Events by category
                let categories = Array(analyticsSummary?.eventsByCategory.keys ?? [])
                let category = categories[indexPath.row]
                let count = analyticsSummary?.eventsByCategory[category] ?? 0
                content.text = getEventCategoryLabel(category)
                content.secondaryText = "\(count)"
            case 2:
                // Metrics
                let metrics = Array(analyticsSummary?.metrics.keys ?? [])
                let metric = metrics[indexPath.row]
                let value = analyticsSummary?.metrics[metric] ?? ""
                content.text = getMetricLabel(metric)
                content.secondaryText = value
            default:
                break
            }
            
            cell.contentConfiguration = content
            return cell
        }
    }
    
    func tableView(_ tableView: UITableView, titleForHeaderInSection section: Int) -> String? {
        if segmentedControl.selectedSegmentIndex == 0 {
            return section == 0 ? "統計" : "最近のクラッシュ"
        } else {
            switch section {
            case 0: return "サマリー"
            case 1: return "カテゴリー別イベント"
            case 2: return "メトリクス"
            default: return nil
            }
        }
    }
}

// MARK: - UITableViewDelegate

extension DiagnosticsViewController: UITableViewDelegate {
    
    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        tableView.deselectRow(at: indexPath, animated: true)
        
        if segmentedControl.selectedSegmentIndex == 0 && indexPath.section == 1 {
            // Show crash report detail
            let report = crashReports[indexPath.row]
            let detailVC = CrashReportDetailViewController(report: report)
            navigationController?.pushViewController(detailVC, animated: true)
        }
    }
}

// MARK: - Helper Methods

private func getCrashTypeLabel(_ type: CrashType) -> String {
    switch type {
    case .nativeCrash: return "ネイティブクラッシュ"
    case .exception: return "例外"
    case .outOfMemory: return "メモリ不足"
    case .anr: return "応答なし"
    case .networkError: return "ネットワークエラー"
    case .filterError: return "フィルターエラー"
    case .other: return "その他"
    }
}

private func getEventCategoryLabel(_ category: EventCategory) -> String {
    switch category {
    case .lifecycle: return "ライフサイクル"
    case .action: return "アクション"
    case .performance: return "パフォーマンス"
    case .error: return "エラー"
    case .feature: return "機能"
    }
}

private func getMetricLabel(_ name: String) -> String {
    switch name {
    case "ads_blocked": return "ブロックした広告"
    case "bytes_saved": return "節約したデータ"
    case "vpn_connections": return "VPN接続回数"
    case "custom_rules": return "カスタムルール"
    default: return name
    }
}

private func formatDate(_ date: Date) -> String {
    let formatter = DateFormatter()
    formatter.dateFormat = "MM/dd HH:mm"
    formatter.locale = Locale(identifier: "ja_JP")
    return formatter.string(from: date)
}

// MARK: - Custom Cell

class DiagnosticsDetailCell: UITableViewCell {
    
    private let statsLabel = UILabel()
    private let detailsStackView = UIStackView()
    
    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)
        setupUI()
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
    private func setupUI() {
        selectionStyle = .none
        
        statsLabel.font = .preferredFont(forTextStyle: .headline)
        statsLabel.numberOfLines = 0
        
        detailsStackView.axis = .vertical
        detailsStackView.spacing = 4
        
        let stackView = UIStackView(arrangedSubviews: [statsLabel, detailsStackView])
        stackView.axis = .vertical
        stackView.spacing = 8
        stackView.translatesAutoresizingMaskIntoConstraints = false
        
        contentView.addSubview(stackView)
        
        NSLayoutConstraint.activate([
            stackView.topAnchor.constraint(equalTo: contentView.topAnchor, constant: 12),
            stackView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 16),
            stackView.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -16),
            stackView.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -12)
        ])
    }
    
    func configure(with stats: CrashStatistics?) {
        guard let stats = stats else { return }
        
        let color: UIColor = stats.totalCrashes == 0 ? .systemGreen : .systemRed
        statsLabel.text = "合計クラッシュ数: \(stats.totalCrashes)"
        statsLabel.textColor = color
        
        detailsStackView.arrangedSubviews.forEach { $0.removeFromSuperview() }
        
        for (type, count) in stats.crashesByType {
            let label = UILabel()
            label.font = .preferredFont(forTextStyle: .caption1)
            label.text = "\(getCrashTypeLabel(type)): \(count)"
            label.textColor = .secondaryLabel
            detailsStackView.addArrangedSubview(label)
        }
    }
}