//! 複雜度預算系統（Complexity Budget）
//!
//! 負責管理推理圖的複雜度上限與計算。

use serde::{Deserialize, Serialize};

/// 複雜度預算系統
///
/// 用於追蹤和控制推理圖的總複雜度。
///
/// # 複雜度公式
/// `Complex = a × k × m`
/// - `a`: 當前層數 - 未定義層數
/// - `k`: 常數（預設 1.0）
/// - `m`: 該節點的分散數量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityBudget {
    /// 常數（預設 1.0）
    pub k: f64,
    /// 最大複雜度上限（預設 100.0）
    pub max_complexity: f64,
    /// 目前複雜度
    pub current_complexity: f64,
}

impl Default for ComplexityBudget {
    fn default() -> Self {
        ComplexityBudget::new()
    }
}

impl ComplexityBudget {
    /// 建立新的複雜度預算（預設值）
    ///
    /// # 範例
    /// ```
    /// let budget = ComplexityBudget::new();
    /// assert_eq!(budget.k, 1.0);
    /// assert_eq!(budget.max_complexity, 100.0);
    /// assert_eq!(budget.current_complexity, 0.0);
    /// ```
    pub fn new() -> Self {
        ComplexityBudget {
            k: 1.0,
            max_complexity: 100.0,
            current_complexity: 0.0,
        }
    }

    /// 建立自訂參數的複雜度預算
    pub fn new_with(k: f64, max_complexity: f64) -> Self {
        ComplexityBudget {
            k,
            max_complexity,
            current_complexity: 0.0,
        }
    }

    /// 計算複雜度
    ///
    /// 公式：Complex = a × k × m
    /// - `a`: 當前層數 - 未定義層數
    /// - `m`: 該節點的分散數量
    ///
    /// # 範例
    /// ```
    /// let budget = ComplexityBudget::new();
    /// let complex = budget.calculate(3, 2); // 3 * 1.0 * 2 = 6.0
    /// assert!((complex - 6.0).abs() < 0.001);
    /// ```
    pub fn calculate(&self, a: i32, m: i32) -> f64 {
        (a as f64) * self.k * (m as f64)
    }

    /// 檢查是否超出預算
    ///
    /// # 範例
    /// ```
    /// let budget = ComplexityBudget::new_with(1.0, 10.0);
    /// budget.current_complexity = 5.0;
    /// assert!(!budget.is_over_budget());
    /// budget.current_complexity = 15.0;
    /// assert!(budget.is_over_budget());
    /// ```
    pub fn is_over_budget(&self) -> bool {
        self.current_complexity > self.max_complexity
    }

    /// 加入複雜度
    ///
    /// # 引數
    /// - `amount`: 要加入的複雜度量
    pub fn add_complexity(&mut self, amount: f64) {
        self.current_complexity += amount;
    }

    /// 減少複雜度
    ///
    /// # 引數
    /// - `amount`: 要減少的複雜度量
    pub fn remove_complexity(&mut self, amount: f64) {
        self.current_complexity = (self.current_complexity - amount).max(0.0);
    }

    /// 重置目前複雜度
    pub fn reset(&mut self) {
        self.current_complexity = 0.0;
    }

    /// 取得剩餘複雜度預算
    pub fn remaining(&self) -> f64 {
        (self.max_complexity - self.current_complexity).max(0.0)
    }

    /// 檢查是否可以加入指定複雜度
    pub fn can_add(&self, amount: f64) -> bool {
        self.current_complexity + amount <= self.max_complexity
    }
}

/// 閾值觸發系統（Threshold Gate）
///
/// 當複雜度或信心度超過閾值時觸發收斂。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdGate {
    /// 信心度權重（預設 0.6）
    pub confidence_weight: f64,
    /// 數量權重
    pub quantity_weight: f64,
    /// 深度權重
    pub depth_weight: f64,
    /// 觸發門檻
    pub threshold: f64,
}

impl Default for ThresholdGate {
    fn default() -> Self {
        ThresholdGate::new()
    }
}

impl ThresholdGate {
    /// 建立新的閾值觸發系統（預設值）
    ///
    /// # 範例
    /// ```
    /// let gate = ThresholdGate::new();
    /// assert_eq!(gate.confidence_weight, 0.6);
    /// assert_eq!(gate.threshold, 50.0);
    /// ```
    pub fn new() -> Self {
        ThresholdGate {
            confidence_weight: 0.6,
            quantity_weight: 0.3,
            depth_weight: 0.1,
            threshold: 50.0,
        }
    }

    /// 建立自訂參數的閾值觸發系統
    pub fn new_with(threshold: f64, confidence_weight: f64) -> Self {
        ThresholdGate {
            confidence_weight,
            quantity_weight: 0.3,
            depth_weight: 0.1,
            threshold,
        }
    }

    /// 判斷是否應該觸發收斂
    ///
    /// 當 Complex > Threshold 或 confidence > 0.8 時觸發收斂
    ///
    /// # 引數
    /// - `complexity`: 當前複雜度
    /// - `confidence`: 信心度（0.0 ~ 1.0）
    ///
    /// # 範例
    /// ```
    /// let gate = ThresholdGate::new();
    /// // 複雜度過高
    /// assert!(gate.should_converge(60.0, 0.5));
    /// // 信心度過高
    /// assert!(gate.should_converge(30.0, 0.85));
    /// // 兩者都低於閾值
    /// assert!(!gate.should_converge(30.0, 0.5));
    /// ```
    pub fn should_converge(&self, complexity: f64, confidence: f64) -> bool {
        complexity > self.threshold || confidence > 0.8
    }

    /// 計算收斂分數
    ///
    /// 分數 = confidence_weight * confidence + quantity_weight * quantity + depth_weight * depth
    ///
    /// # 引數
    /// - `confidence`: 信心度
    /// - `quantity`: 數量
    /// - `depth`: 深度
    pub fn convergence_score(&self, confidence: f64, quantity: i32, depth: i32) -> f64 {
        self.confidence_weight * confidence
            + self.quantity_weight * (quantity as f64)
            + self.depth_weight * (depth as f64)
    }

    /// 檢查是否應該根據分數收斂
    ///
    /// # 引數
    /// - `score`: 收斂分數
    pub fn should_prune_by_score(&self, score: f64) -> bool {
        score < self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_budget_new() {
        let budget = ComplexityBudget::new();
        assert_eq!(budget.k, 1.0);
        assert_eq!(budget.max_complexity, 100.0);
        assert_eq!(budget.current_complexity, 0.0);
    }

    #[test]
    fn test_complexity_budget_calculate() {
        let budget = ComplexityBudget::new();
        let complex = budget.calculate(3, 2);
        assert!((complex - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_complexity_budget_is_over_budget() {
        let budget = ComplexityBudget::new_with(1.0, 10.0);
        assert!(!budget.is_over_budget());

        let mut b2 = ComplexityBudget::new_with(1.0, 10.0);
        b2.current_complexity = 15.0;
        assert!(b2.is_over_budget());
    }

    #[test]
    fn test_complexity_budget_add_remove() {
        let mut budget = ComplexityBudget::new();
        budget.add_complexity(10.0);
        assert!((budget.current_complexity - 10.0).abs() < 0.001);
        budget.remove_complexity(5.0);
        assert!((budget.current_complexity - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_complexity_budget_remaining() {
        let mut budget = ComplexityBudget::new_with(1.0, 100.0);
        budget.current_complexity = 30.0;
        assert!((budget.remaining() - 70.0).abs() < 0.001);
    }

    #[test]
    fn test_complexity_budget_can_add() {
        let budget = ComplexityBudget::new_with(1.0, 100.0);
        assert!(budget.can_add(50.0));   // 0 + 50 <= 100
        assert!(budget.can_add(60.0));   // 0 + 60 <= 100
        // 當已有 50 複雜度時，再加 60 就超出預算了
        let mut budget2 = ComplexityBudget::new_with(1.0, 100.0);
        budget2.current_complexity = 50.0;
        assert!(!budget2.can_add(60.0)); // 50 + 60 > 100
    }

    #[test]
    fn test_threshold_gate_new() {
        let gate = ThresholdGate::new();
        assert_eq!(gate.confidence_weight, 0.6);
        assert_eq!(gate.threshold, 50.0);
    }

    #[test]
    fn test_threshold_gate_should_converge() {
        let gate = ThresholdGate::new();

        // 複雜度過高 -> 應該收斂
        assert!(gate.should_converge(60.0, 0.5));

        // 信心度過高 -> 應該收斂
        assert!(gate.should_converge(30.0, 0.85));

        // 兩者都低 -> 不應該收斂
        assert!(!gate.should_converge(30.0, 0.5));

        // 邊界：複雜度剛好等於閾值 -> 不收斂（要大於）
        assert!(!gate.should_converge(50.0, 0.5));

        // 邊界：信心度剛好等於 0.8 -> 不收斂（要大於）
        assert!(!gate.should_converge(30.0, 0.8));
    }

    #[test]
    fn test_threshold_gate_convergence_score() {
        let gate = ThresholdGate::new();
        let score = gate.convergence_score(0.8, 5, 3);
        // 0.6 * 0.8 + 0.3 * 5 + 0.1 * 3 = 0.48 + 1.5 + 0.3 = 2.28
        assert!((score - 2.28).abs() < 0.001);
    }

    #[test]
    fn test_threshold_gate_should_prune_by_score() {
        let gate = ThresholdGate::new();
        assert!(gate.should_prune_by_score(30.0));
        assert!(!gate.should_prune_by_score(60.0));
    }
}
